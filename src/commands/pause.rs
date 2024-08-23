use std::sync::Arc;

use serenity::all::GuildId;
use songbird::Songbird;
use tracing::error;

use super::{Context, Error};

/// Pausa a música que está tocando
#[poise::command(slash_command, guild_only, subcommands("on", "off"))]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Guild ID is not available");
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.");

    if let Err(err) = pause_song(manager, guild_id, false).await {
        ctx.reply(err.to_string()).await?;
    } else {
        ctx.reply("Música pausada").await?;
    }

    Ok(())
}

/// Pausa a música que está tocando
#[poise::command(prefix_command, slash_command)]
async fn on(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Guild ID is not available");
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.");

    if let Err(err) = pause_song(manager, guild_id, false).await {
        ctx.reply(err.to_string()).await?;
    } else {
        ctx.reply("Música pausada").await?;
    }

    Ok(())
}

/// Continua a tocar a música de onde parou
#[poise::command(prefix_command, slash_command)]
async fn off(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Guild ID is not available");
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.");

    if let Err(err) = pause_song(manager, guild_id, true).await {
        ctx.reply(err.to_string()).await?;
    } else {
        ctx.reply("Música despausada").await?;
    }

    Ok(())
}

pub(super) async fn pause_song(
    manager: Arc<Songbird>,
    guild_id: GuildId,
    unpause: bool,
) -> Result<(), anyhow::Error> {
    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => return Err(anyhow::anyhow!("Não estou em nenhum canal")),
    };
    let handler = handler_lock.lock().await;

    if unpause {
        if let Err(cause) = handler.queue().resume() {
            error!(%cause, "failed to unpause");
            return Err(anyhow::anyhow!("Não consegui despausar"));
        }
        Ok(())
    } else {
        if let Err(cause) = handler.queue().pause() {
            error!(%cause, "failed to pause");
            return Err(anyhow::anyhow!("Não consegui pausar"));
        }
        Ok(())
    }
}
