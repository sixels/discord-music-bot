use std::sync::Arc;

use serenity::{
    all::{CommandInteraction, CommandOptionType, GuildId},
    async_trait,
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};

use songbird::Songbird;
use tracing::error;

use crate::commands::common;

/// Pause the current song
pub struct Pause;

#[async_trait]
impl super::Command for Pause {
    fn name(&self) -> String {
        String::from("pause")
    }

    fn register(&self, cmd: CreateCommand) -> CreateCommand {
        cmd.description("Pausa a música que está tocando")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "on",
                    "Pausa a música atual",
                )
                .required(false)
                .set_autocomplete(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "off",
                    "Continua a tocar a música de onde parou",
                )
                .required(false)
                .set_autocomplete(true),
            )
    }

    async fn run(&self, ctx: &Context, cmd: &CommandInteraction) {
        let options = cmd.data.options();
        let unpause = common::get_option(&options, "off")
            .map(|_| true)
            .unwrap_or(false);

        let guild_id = common::get_guild_id(ctx, cmd);
        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.");

        match pause(manager, guild_id, unpause).await {
            Ok(message) => common::respond(ctx, cmd, message),
            Err(err) => common::respond(ctx, cmd, err),
        }
        .await;
    }
}

pub(crate) async fn pause(
    manager: Arc<Songbird>,
    guild_id: GuildId,
    unpause: bool,
) -> Result<String, String> {
    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => return Err("Não estou em nenhum canal".to_string()),
    };
    let handler = handler_lock.lock().await;

    if unpause {
        if let Err(cause) = handler.queue().resume() {
            error!(%cause, "failed to unpause");
            return Err("Não consegui despausar".to_string());
        }
        Ok("Música despausada".to_string())
    } else {
        if let Err(cause) = handler.queue().pause() {
            error!(%cause, "failed to pause");
            return Err("Não consegui pausar".to_string());
        }
        Ok("Música pausada".to_string())
    }
}
