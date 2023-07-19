use std::sync::Arc;

use serenity::{
    all::{CommandInteraction, CommandOptionType, GuildId, ResolvedValue},
    async_trait,
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    client::Context,
    model::Colour,
};

use songbird::Songbird;

use crate::commands::common;

use super::play::SongMetadataKey;

/// List the songs in the queue
pub struct List;

#[async_trait]
impl super::Command for List {
    fn name(&self) -> String {
        String::from("list")
    }

    fn register(&self, cmd: CreateCommand) -> CreateCommand {
        cmd.description("Lista as músicas que estão na fila")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "page",
                    "Mostra outra página da lista",
                )
                .min_int_value(1)
                .required(false)
                .set_autocomplete(false),
            )
    }

    async fn run(&self, ctx: Context, cmd: CommandInteraction) {
        let options = cmd.data.options();
        let option_page = common::get_option(&options, "page")
            .and_then(|val| {
                if let ResolvedValue::Integer(page) = val {
                    Some((*page) as usize)
                } else {
                    None
                }
            })
            .unwrap_or(1);

        let guild_id = common::get_guild_id(&ctx, &cmd);
        let manager = songbird::get(&ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.");

        match list(manager, guild_id, option_page - 1).await {
            Ok(message) => {
                let _ = cmd
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .add_embed(
                                    CreateEmbed::new()
                                        .field("LISTA DE REPRODUÇÃO ATUAL", message, false)
                                        .colour(Colour::BLUE),
                                )
                                .ephemeral(true),
                        ),
                    )
                    .await
                    .ok();
            }
            Err(err) => common::respond(&ctx, &cmd, err).await,
        }
    }
}

const TRACK_LIST_SIZE: usize = 10;

pub(crate) async fn list(
    manager: Arc<Songbird>,
    guild_id: GuildId,
    page: usize,
) -> Result<String, String> {
    let Some(handler_lock) =  manager.get(guild_id) else {
        return Err("Não estou em nenhum canal".to_string())
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
