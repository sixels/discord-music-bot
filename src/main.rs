use anyhow::anyhow;
use commands::{Join, Leave, Play};
use reqwest::Client as HttpClient;
use serenity::{
    all::Interaction,
    async_trait,
    builder::CreateCommand,
    client::Context,
    model::{gateway::Ready, prelude::GuildId},
    prelude::{EventHandler, TypeMapKey},
};
use service::{CreateService, Service};
use shuttle_secrets::SecretStore;
use tracing::{error, info};

use crate::commands::Command;

mod commands;
mod events;
mod service;

struct Handler {
    guild_id: String,
    commands: Vec<Box<dyn Command + Send + Sync + 'static>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(cmd) = interaction {
            let command_name = cmd.data.name.as_str();
            let command = self.commands.iter().find(|c| c.name() == command_name);

            if let Some(command) = command {
                info!("handling {} command", command.name());
                command.run(&ctx, &cmd).await;
            } else {
                error!("invalid command passed: {}", command_name)
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
                self.commands
                    .iter()
                    .map(|c| c.register(CreateCommand::new(c.name())))
                    .collect(),
            )
            .await
        {
            panic!("failed to register commands: {cause}")
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

    let service = CreateService::new(token, guild_id)
        .with_command(Join)
        .with_command(Leave)
        .with_command(Play)
        .create()
        .await;
    Ok(service)
}
