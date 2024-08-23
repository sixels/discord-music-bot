use std::sync::Arc;

use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{all::GuildId, builder::CreateEmbed, model::Colour};
use songbird::Songbird;

use super::{play::SongMetadataKey, Context, Error};

const TRACK_LIST_SIZE: usize = 10;

/// Lista as músicas na fila de reprodução
#[poise::command(slash_command, guild_only)]
pub async fn list(
    ctx: Context<'_>,
    #[description = "Escolhe a página da lista"] option_page: Option<String>,
) -> Result<(), Error> {
    let option_page = option_page
        .and_then(|page| page.parse::<usize>().ok())
        .unwrap_or(1);

    let guild_id = ctx.guild_id().ok_or(anyhow::anyhow!("Not in a guild"))?;
    let manager = songbird::get(&ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.");

    match show_list(manager, guild_id, option_page - 1).await {
        Ok(message) => {
            let response = CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .field("LISTA DE REPRODUÇÃO ATUAL", message, false)
                        .colour(Colour::BLUE),
                )
                .ephemeral(true);
            let _ = ctx.send(response).await;

            Ok(())
        }
        Err(err) => {
            ctx.reply(err.to_string()).await?;
            Ok(())
        }
    }
}

pub(super) async fn show_list(
    manager: Arc<Songbird>,
    guild_id: GuildId,
    page: usize,
) -> Result<String, String> {
    let Some(handler_lock) = manager.get(guild_id) else {
        return Err("Não estou em nenhum canal".to_string());
    };

    let handler = handler_lock.lock().await;
    let start_at = page * TRACK_LIST_SIZE;

    let q = handler.queue();

    let list_size = q.len().min(TRACK_LIST_SIZE);

    if list_size == 0 {
        return Err("A lista está vazia".into());
    }

    let mut list = Vec::with_capacity(list_size);

    for (i, track) in q
        .current_queue()
        .iter()
        .skip(start_at)
        .take(TRACK_LIST_SIZE)
        .enumerate()
    {
        let map = track.typemap().read().await;
        let Some(meta) = map.get::<SongMetadataKey>() else {
            continue;
        };

        let position = i + start_at + 1;

        list.push(format!("{position}. {}", meta.title));
    }

    Ok(list.join("\n"))
}
