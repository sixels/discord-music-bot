use std::sync::Arc;

use serenity::{
    all::{CommandInteraction, CommandOptionType, GuildId, ResolvedValue},
    async_trait,
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};

use songbird::Songbird;
use tracing::info;

use crate::commands::common;

/// Skip some songs
pub struct Skip;

#[async_trait]
impl super::Command for Skip {
    fn name(&self) -> String {
        String::from("skip")
    }

    fn register(&self, cmd: CreateCommand) -> CreateCommand {
        cmd.description("Pula algumas músicas")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "first",
                    "Pula a música atual",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "count",
                        "Pula as n primeiras músicas",
                    )
                    .min_int_value(1)
                    .required(false),
                )
                .required(false)
                .set_autocomplete(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "position",
                    "Pula a música na posição indicada",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "at",
                        "A posição da música a ser pulada",
                    )
                    .required(true)
                    .min_int_value(1),
                )
                .required(false)
                .set_autocomplete(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "range",
                    "Pula as músicas em no intervalo (inclusivo) indicado",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "start",
                        "O começo do intervalo",
                    )
                    .min_int_value(1)
                    .required(true),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "end",
                        "O final do intervalo",
                    )
                    .min_int_value(2)
                    .required(true),
                )
                .required(false)
                .set_autocomplete(true),
            )
    }

    async fn run(&self, ctx: &Context, cmd: &CommandInteraction) {
        let options = cmd.data.options();
        let option_first = common::get_option(&options, "first")
            .and_then(|opt| {
                if let ResolvedValue::SubCommand(sub) = opt {
                    Some(sub)
                } else {
                    None
                }
            })
            .map(|sub| {
                common::get_option(sub, "count")
                    .and_then(|val| {
                        if let ResolvedValue::Integer(count) = val {
                            Some(*count as usize)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(1)
            });
        let option_position = common::get_option(&options, "position")
            .and_then(|opt| {
                if let ResolvedValue::SubCommand(sub) = opt {
                    Some(sub)
                } else {
                    None
                }
            })
            .and_then(|sub| {
                common::get_option(sub, "at").and_then(|val| {
                    if let ResolvedValue::Integer(at) = val {
                        Some(*at as usize)
                    } else {
                        None
                    }
                })
            });
        let option_range = common::get_option(&options, "range")
            .and_then(|opt| {
                if let ResolvedValue::SubCommand(sub) = opt {
                    Some(sub)
                } else {
                    None
                }
            })
            .map(|sub| {
                let start = common::get_option(sub, "start").and_then(|val| {
                    if let ResolvedValue::Integer(start) = val {
                        Some(*start as usize)
                    } else {
                        None
                    }
                });
                let end = common::get_option(sub, "end").and_then(|val| {
                    if let ResolvedValue::Integer(start) = val {
                        Some(*start as usize)
                    } else {
                        None
                    }
                });
                (start.unwrap_or(1), end.unwrap_or(usize::MAX))
            });

        let mut range = (0, 0);
        if let Some(r) = option_range {
            range = r
        }
        if let Some(at) = option_position {
            let at = at - 1;
            range = (at, at);
        }
        if let Some(count) = option_first {
            range = (0, count - 1)
        }

        let guild_id = common::get_guild_id(ctx, cmd);
        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.");

        match skip(manager, guild_id, range).await {
            Ok(message) => common::respond(ctx, cmd, message),
            Err(err) => common::respond(ctx, cmd, err),
        }
        .await
    }
}

async fn skip(
    manager: Arc<Songbird>,
    guild_id: GuildId,
    range: (usize, usize),
) -> Result<String, String> {
    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => return Err("Não estou em nenhum canal".to_string()),
    };
    let handler = handler_lock.lock().await;
    let q = handler.queue();

    let (start, end) = (range.0, range.1.min(q.len()));
    if end < start {
        return Err("Intervalo inválido".to_string());
    }

    q.modify_queue(|q| {
        info!("queue.len = {}", q.len());

        assert!(end >= start);

        let mut removed_playing = false;
        for i in start..=end {
            if let Some(track) = q.remove(i) {
                removed_playing |= i == 0;
                track.handle().stop().ok();
            }
        }

        // play the next music if the one that was playing got removed
        if removed_playing {
            if let Some(track) = q.get_mut(0) {
                track.play().ok();
            }
        }
    });

    Ok("Pulando algumas músicas".to_string())
}
