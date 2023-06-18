use anyhow::anyhow;
use serenity::{
    async_trait,
    client::{Client, Context},
    framework::StandardFramework,
    model::{
        application::interaction::Interaction,
        gateway::Ready,
        prelude::{command::CommandType, GuildId},
    },
    prelude::{EventHandler, GatewayIntents},
};
// use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use songbird::SerenityInit;
use tracing::{error, info};

mod commands;

struct Handler {
    guild_id: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "join" => {
                    info!("handling join command");
                    commands::join(&ctx, &command).await;
                }
                "leave" => {
                    info!("handling leave command");
                    commands::leave(&ctx, &command).await;
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
        let _ = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command
                        .name("join")
                        .description("Join your current voice channel")
                })
                .create_application_command(|command| {
                    command.name("leave").description("Leve the voice channel")
                })
        })
        .await;
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };
    let guild_id = if let Some(token) = secret_store.get("GUILD_ID") {
        token
    } else {
        return Err(anyhow!("'GUILD_ID' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::non_privileged();
    let framework = StandardFramework::new();

    let handler = Handler { guild_id };
    let client = Client::builder(&token, intents)
        .event_handler(handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    Ok(client.into())
}
