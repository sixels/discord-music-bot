use serenity::{all::CommandInteraction, async_trait, builder::CreateCommand, client::Context};
use tracing::{error, info};

use super::respond;

/// Join your current voice channel
pub struct Join;

#[async_trait]
impl super::Command for Join {
    fn name() -> String {
        String::from("join")
    }

    fn register() -> CreateCommand {
        CreateCommand::new(Self::name()).description("Join your current voice channel")
    }

    async fn run(ctx: &Context, cmd: &CommandInteraction) {
        let (guild_id, channel_id) = {
            let guild = ctx.cache.guild(cmd.guild_id.unwrap()).unwrap();

            let channel_id = guild
                .voice_states
                .get(&cmd.user.id)
                .and_then(|voice_state| voice_state.channel_id);

            (guild.id, channel_id)
        };

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
            }
        };

        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        let channel_name = connect_to
            .name(&ctx.http)
            .await
            .unwrap_or(String::from("?"));
        info!(channel_id = connect_to.0, ?channel_name, "joining channel");

        if let Err(cause) = manager.join(guild_id, connect_to).await {
            error!(%cause, "failed to join channel");
            respond(
                ctx,
                cmd,
                "Não consegui entrar no canal, tente novamente mais tarde",
            )
            .await;
            return;
        };

        respond(ctx, cmd, &format!("Entrou em {channel_name}")).await;
    }
}
