use tracing::error;

use super::{Context, Error};

/// Leave the voice channel

/// Sai do canal de voz
#[poise::command(slash_command, guild_only)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(anyhow::anyhow!("Not in a guild"))?;

    let manager = songbird::get(&ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if manager.get(guild_id).is_none() {
        ctx.reply("Não estou em nenhum canal").await?;
        return Err(anyhow::anyhow!("not in a channel"));
    }

    if let Err(cause) = manager.remove(guild_id).await {
        error!(%cause, "failed to leave channel");
        ctx.reply("Não consegui sair do canal").await?;
    } else {
        ctx.reply("Saindo do canal").await?;
    }

    Ok(())
}
