use serenity::{all::CommandInteraction, async_trait, builder::CreateCommand, client::Context};
use tracing::error;

use super::common;

/// Leave the voice channel
pub struct Leave;

#[async_trait]
impl super::Command for Leave {
    fn name(&self) -> String {
        String::from("leave")
    }
    fn register(&self, cmd: CreateCommand) -> CreateCommand {
        cmd.description("Sai do canal de voz")
    }

    async fn run(&self, ctx: Context, cmd: CommandInteraction) {
        let guild_id = common::get_guild_id(&ctx, &cmd);
        let manager = songbird::get(&ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        if manager.get(guild_id).is_none() {
            common::respond(&ctx, &cmd, "Não estou em nenhum canal").await;
            return;
        }

        if let Err(cause) = manager.remove(guild_id).await {
            error!(%cause, "failed to leave channel");
            common::respond(&ctx, &cmd, "Não consegui sair do canal").await;
        } else {
            common::respond(&ctx, &cmd, "Saindo do canal").await;
        }
    }
}
