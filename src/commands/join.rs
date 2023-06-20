use serenity::{
    async_trait, builder::CreateApplicationCommand, client::Context,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
};
use tracing::{error, info};

use super::respond;

/// Join your current voice channel
pub struct Join;

#[async_trait]
impl super::Command for Join {
    fn name() -> String {
        String::from("join")
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name(Self::name())
            .description("Join your current voice channel")
    }

    async fn run(ctx: &Context, cmd: &ApplicationCommandInteraction) {
        let guild = ctx.cache.guild(cmd.guild_id.unwrap()).unwrap();
        let guild_id = guild.id;

        let channel_id = guild
            .voice_states
            .get(&cmd.user.id)
            .and_then(|voice_state| voice_state.channel_id);

        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                // check_msg(cmd.reply(ctx, "Not in a voice channel").await);
                error!("not in a voice channel");
                respond(
                    ctx,
                    cmd,
                    "Você deve entrar em um canal de voz antes de usar o comando",
                )
                .await;
                return;
                // return Ok(());
            }
        };

        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        let channel_name = connect_to.name(&ctx.cache).await;
        info!(channel_id = connect_to.0, ?channel_name, "joining channel");

        let (_handler, result) = manager.join(guild_id, connect_to).await;
        if let Err(cause) = result {
            error!(%cause, "failed to join channel");
            respond(
                ctx,
                cmd,
                "Não consegui entrar no canal, tente novamente mais tarde",
            )
            .await;
            return;
        }

        let channel_name_str = if let Some(name) = channel_name.as_deref() {
            name
        } else {
            "?"
        };
        respond(ctx, cmd, &format!("Entrou em {channel_name_str}")).await;
    }
}
