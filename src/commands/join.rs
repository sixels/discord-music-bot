use std::sync::Arc;

use serenity::all::ChannelId;
use serenity::{all::CommandInteraction, async_trait, builder::CreateCommand, client::Context};
use songbird::{Call, Songbird};
use tokio::sync::Mutex;
use tracing::{error, info};

use super::common;

/// Join your current voice channel
pub struct Join;

#[async_trait]
impl super::Command for Join {
    fn name(&self) -> String {
        String::from("join")
    }
    fn register(&self, cmd: CreateCommand) -> CreateCommand {
        cmd.description("Entra no canal de voz que você está")
    }

    async fn run(&self, ctx: Context, cmd: CommandInteraction) {
        let manager = songbird::get(&ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.");

        match join_channel(manager, &ctx, &cmd).await {
            Ok((_, channel_id)) => {
                let channel_name = channel_id
                    .name(&ctx.http)
                    .await
                    .unwrap_or(String::from("?"));
                info!(channel_id = channel_id.0, ?channel_name, "joining channel");
            }
            Err(e) => {
                common::respond(&ctx, &cmd, &e).await;
            }
        }
    }
}

pub async fn join_channel(
    manager: Arc<Songbird>,
    ctx: &Context,
    cmd: &CommandInteraction,
) -> Result<(Arc<Mutex<Call>>, ChannelId), String> {
    let (guild_id, channel_id) = common::get_guild_and_channel(ctx, cmd);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            // check_msg(cmd.reply(ctx, "Not in a voice channel").await);
            error!("not in a voice channel");
            return Err(
                "Você deve entrar em um canal de voz antes de usar esse comando".to_string(),
            );
        }
    };

    manager.join(guild_id, connect_to).await.map_or_else(
        |cause| {
            error!(%cause, "failed to join channel");
            Err("Não consegui entrar no canal. Tente novamente mais tarde".to_string())
        },
        |handler| Ok((handler, connect_to)),
    )
}
