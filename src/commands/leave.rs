use serenity::{
    async_trait, builder::CreateApplicationCommand, client::Context,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
};
use tracing::error;

use super::respond;

/// Leave the voice channel
pub struct Leave;

#[async_trait]
impl super::Command for Leave {
    fn name() -> String {
        String::from("leave")
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name(Self::name())
            .description("Leave the voice channel")
    }

    async fn run(ctx: &Context, cmd: &ApplicationCommandInteraction) {
        let guild = ctx.cache.guild(cmd.guild_id.unwrap()).unwrap();
        let guild_id = guild.id;

        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        if manager.get(guild_id).is_some() {
            if let Err(cause) = manager.remove(guild_id).await {
                error!(%cause, "failed to leave channel");
                respond(ctx, cmd, "Não consegui sair do canal").await;
            } else {
                respond(ctx, cmd, "Saiu do canal").await;
            }
        } else {
            respond(ctx, cmd, "Não estou em nenhum canal").await;
        }
    }
}
