use std::sync::Arc;

use serenity::all::ChannelId;
use serenity::model::guild::Guild;
use songbird::{Call, Songbird};
use tokio::sync::Mutex;
use tracing::{error, info};

use super::{Context, Error};

/// Entra no canal de voz que você está
#[poise::command(slash_command, guild_only)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild()
        .ok_or(anyhow::anyhow!("Not in a guild"))?
        .clone();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or(anyhow::anyhow!("Songbird Voice client not found"))?;

    match join_channel(manager, &ctx, &guild).await {
        Ok((_, channel_id)) => {
            let channel_name = channel_id
                .name(&ctx.http())
                .await
                .unwrap_or(String::from("?"));

            info!(
                channel_id = channel_id.get(),
                ?channel_name,
                "joining channel"
            );
            ctx.reply(format!("Entrando em **{channel_name}**")).await?;
        }
        Err(e) => {
            ctx.reply(e.to_string()).await?;
        }
    };

    Ok(())
}

pub(super) async fn join_channel(
    manager: Arc<Songbird>,
    ctx: &Context<'_>,
    guild: &Guild,
) -> Result<(Arc<Mutex<Call>>, ChannelId), Error> {
    let user_voice_channel = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match user_voice_channel {
        Some(channel) => channel,
        None => {
            // check_msg(cmd.reply(ctx, "Not in a voice channel").await);
            error!("not in a voice channel");
            return Err(anyhow::anyhow!(
                "Você deve entrar em um canal de voz antes de usar esse comando"
            ));
        }
    };

    manager.join(guild.id, connect_to).await.map_or_else(
        |cause| {
            error!(%cause, "failed to join channel");
            Err(anyhow::anyhow!(
                "Não consegui entrar no canal. Tente novamente mais tarde".to_string()
            ))
        },
        |handler| Ok((handler, connect_to)),
    )
}
