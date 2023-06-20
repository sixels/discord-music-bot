use anyhow::anyhow;
use serenity::{
    async_trait,
    client::{Client, Context},
    framework::StandardFramework,
    model::{application::interaction::Interaction, gateway::Ready, prelude::GuildId},
    prelude::{EventHandler, GatewayIntents},
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
        if let Interaction::ApplicationCommand(command) = interaction {
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

        if let Ok(cmds) = guild_id.get_application_commands(&ctx.http).await {
            info!("deleting old commands");
            for cmd in cmds {
                guild_id
                    .delete_application_command(&ctx.http, cmd.id)
                    .await
                    .ok();
            }
        }

        if let Err(cause) = guild_id
            .set_application_commands(
                &ctx.http,
                |commands: &mut serenity::builder::CreateApplicationCommands| {
                    commands::register_command::<commands::Join>(commands);
                    commands::register_command::<commands::Leave>(commands);
                    commands::register_command::<commands::Play>(commands);

                    commands.create_application_command(|command| {
                        command
                            .name("pause")
                            .description("Pause the current playing music")
                    })
                },
            )
            .await
        {
            panic!("could not register commands: {cause}")
        }
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
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
