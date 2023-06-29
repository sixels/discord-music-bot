use anyhow::anyhow;
use reqwest::Client as HttpClient;
use serenity::{
    all::Interaction,
    async_trait,
    client::{Client, Context},
    framework::StandardFramework,
    model::{gateway::Ready, prelude::GuildId},
    prelude::{EventHandler, GatewayIntents, TypeMapKey},
};
use shuttle_secrets::SecretStore;
use songbird::SerenityInit;
use tracing::{error, info};

use crate::commands::Command;

mod commands;

struct Handler {
    guild_id: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            match command.data.name.as_str() {
                "join" => {
                    info!("handling join command");
                    commands::Join::run(&ctx, &command).await;
                }
                "leave" => {
                    info!("handling leave command");
                    commands::Leave::run(&ctx, &command).await;
                }
                "play" => {
                    info!("handling play command");
                    commands::Play::run(&ctx, &command).await;
                }
                cmd => {
                    error!("invalid command passed: {}", cmd)
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guild_id = GuildId(self.guild_id.parse().unwrap());

        if let Ok(cmds) = guild_id.get_commands(&ctx.http).await {
            info!("deleting old commands");
            for cmd in cmds {
                guild_id.delete_command(&ctx.http, cmd.id).await.ok();
            }
        }

        if let Err(cause) = guild_id
            .set_commands(
                &ctx.http,
                vec![
                    commands::Join::register(),
                    commands::Leave::register(),
                    commands::Play::register(),
                ],
            )
            .await
        {
            panic!("could not register commands: {cause}")
        }
    }
}

struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

type ShuttleResult<T> = Result<T, shuttle_runtime::Error>;

#[shuttle_runtime::main]
async fn serenity(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttleResult<Service> {
    {
        let args = std::env::args();
        let env: std::env::Vars = std::env::vars();
        let env = env.collect::<Vec<_>>();
        info!(?args, ?env, "starting program");
    };

    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    let guild_id_var = if cfg!(debug_assertions) {
        "GUILD_ID"
    } else {
        "GUILD_ID_PROD"
    };

    let guild_id = if let Some(token) = secret_store.get(guild_id_var) {
        token
    } else {
        return Err(anyhow!("'{guild_id_var}' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::non_privileged();
    let framework = StandardFramework::new();

    let handler = Handler { guild_id };
    let client = Client::builder(&token, intents)
        .event_handler(handler)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await
        .expect("Err creating client");

    Ok(Service(client))
}

struct Service(serenity::Client);

#[async_trait]
impl shuttle_runtime::Service for Service {
    async fn bind(mut self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        self.0
            .start()
            .await
            .map_err(shuttle_runtime::CustomError::new)?;
        Ok(())
    }
}
