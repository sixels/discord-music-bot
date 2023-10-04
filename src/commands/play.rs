use std::time::Duration;

use serde::Deserialize;
use serenity::{
    all::{CommandInteraction, CommandOptionType, ResolvedValue, UserId},
    async_trait,
    builder::{
        CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
        CreateInteractionResponseFollowup, CreateMessage,
    },
    client::Context,
    model::Colour,
    prelude::TypeMapKey,
};
use songbird::{
    input::{Compose, YoutubeDl},
    Event,
};
use tracing::{error, info};

use crate::{
    commands::{common, join::join_channel},
    events::track::PlayingSongNotifier,
    service::HttpKey,
};

const PIPED_URL: &str = "https://pipedapi.kavin.rocks";

#[derive(Debug, Deserialize)]
pub struct SearchResults {
    items: Vec<SearchResult>,
}
#[derive(Debug, Deserialize)]
pub struct SearchResult {
    url: String,
    duration: u64,
    title: String,
}

/// Play a song from the given URL
pub struct Play;

#[async_trait]
impl super::Command for Play {
    fn name(&self) -> String {
        String::from("play")
    }

    fn register(&self, cmd: CreateCommand) -> CreateCommand {
        cmd.description("Toca a música que você passar em `query`")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "query",
                    "A URL de algum vídeo",
                )
                .required(true),
            )
    }

    async fn run(&self, ctx: Context, cmd: CommandInteraction) {
        let options = cmd.data.options();
        let option_query = common::get_option(&options, "query").and_then(|opt| {
            if let ResolvedValue::String(val) = opt {
                let query = if val.starts_with("https://") {
                    QueryKind::Url(val)
                } else {
                    QueryKind::Search(val)
                };
                Some(query)
            } else {
                None
            }
        });

        let query = match option_query {
            Some(url) => url,
            None => {
                common::respond(&ctx, &cmd, "Você precisa passar uma URL válida para tocar").await;
                return;
            }
        };
        info!(?query, "preparing to play song");

        let guild_id = common::get_guild_id(&ctx, &cmd);

        let http = {
            let data = ctx.data.read().await;
            data.get::<HttpKey>()
                .cloned()
                .expect("Guaranteed to exist in the typemap.")
        };

        let mut source = match query {
            QueryKind::Url(url) => YoutubeDl::new(http, url.to_string()),
            QueryKind::Search(input) => {
                cmd.defer_ephemeral(&ctx.http).await.ok();

                info!("querying search results");
                let Ok(res) = http
                    .get(format!("{}/search", PIPED_URL))
                    .query(&[
                        ("q", input),
                        ("filter", "videos"),
                        ("filter", "music_songs"),
                        ("filter", "music_videos"),
                    ])
                    .send()
                    .await
                else {
                    common::respond(&ctx, &cmd, "Não consegui pesquisar nenhuma música").await;
                    return;
                };
                // let bytes = res.bytes();

                let Ok(results) = res.json::<SearchResults>().await else {
                    common::respond(&ctx, &cmd, "deu ruim em alguma coisa, tenta de novo").await;
                    return;
                };

                info!("found {} results", results.items.len());

                let display_result = results
                    .items
                    .iter()
                    .enumerate()
                    .map(|(i, result)| {
                        let fmt_duration =
                            humantime::format_duration(Duration::from_secs(result.duration))
                                .to_string();
                        format!("{}. {} ({})", i + 1, result.title, fmt_duration)
                    })
                    .collect::<Vec<String>>();

                let mut followup = CreateInteractionResponseFollowup::new().add_embed(
                    CreateEmbed::new()
                        .field("Resultados", display_result.join("\n"), false)
                        .colour(Colour::RED),
                );

                for (i, result) in results.items.iter().enumerate() {
                    followup = followup
                        .button(CreateButton::new(result.url.clone()).label((i + 1).to_string()))
                }

                info!("creating follow up message");
                let Ok(message) = cmd.create_followup(&ctx.http, followup).await else {
                    common::respond(&ctx, &cmd, "Deu ruim :sob:").await;
                    return;
                };

                let Some(response) = message.await_component_interaction(&ctx).await else {
                    return;
                };

                let video_url = &response.data.custom_id;
                info!("user selected {}", video_url);

                let video_uri = format!("https://www.youtube.com/{video_url}");

                if let Err(cause) = cmd.delete_followup(&ctx.http, message.id).await {
                    error!(%cause, "failed to delete follow up")
                }

                YoutubeDl::new(http, video_uri)
            }
        };

        let meta = source.aux_metadata().await.ok();

        let requester = cmd
            .user
            .global_name
            .clone()
            .unwrap_or(cmd.user.name.clone());

        let song_meta = if let Some(metadata) = meta {
            SongMetadata {
                title: metadata.title.unwrap_or(String::new()),
                duration: metadata.duration.unwrap_or(Duration::ZERO),
                thumbnail: metadata.thumbnail,
                user: requester,
            }
        } else {
            SongMetadata {
                title: String::from(query),
                thumbnail: None,
                duration: Duration::ZERO,
                user: requester,
            }
        };

        cmd.channel_id
            .send_message(
                &ctx.http,
                CreateMessage::new().content(format!(
                    "**{}** adicionou ||{}|| à fila",
                    cmd.user.name, song_meta.title,
                )),
            )
            .await
            .ok();

        let manager = songbird::get(&ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.");

        let handler_lock = match manager.get(guild_id) {
            Some(handler) => handler,
            None => match join_channel(manager, &ctx, &cmd).await {
                Ok((handler, _)) => handler,
                Err(e) => {
                    common::respond(&ctx, &cmd, &e).await;
                    return;
                }
            },
        };
        let mut handler = handler_lock.lock().await;

        let song = handler.enqueue_input(source.into()).await;

        if let Err(cause) = song.add_event(
            Event::Track(songbird::TrackEvent::Play),
            PlayingSongNotifier {
                channel_id: cmd.channel_id,
                http: ctx.http.clone(),
                context: ctx.clone(),
                title: song_meta.title.clone(),
                username: cmd
                    .user
                    .global_name
                    .clone()
                    .unwrap_or(cmd.user.name.clone()),
                thumbnail: song_meta.thumbnail.clone(),
            },
        ) {
            error!(%cause, "failed to create song event")
        }

        let mut typemap = song.typemap().write().await;
        typemap.insert::<SongMetadataKey>(song_meta)
    }
}

#[derive(Debug, Clone)]
enum QueryKind<'s> {
    Url(&'s str),
    Search(&'s str),
}

impl<'s> From<QueryKind<'s>> for String {
    fn from(value: QueryKind<'s>) -> Self {
        match value {
            QueryKind::Url(s) => String::from(s),
            QueryKind::Search(s) => String::from(s),
        }
    }
}

pub struct SongMetadataKey;

pub struct SongMetadata {
    pub title: String,
    pub duration: Duration,
    pub user: String,
    pub thumbnail: Option<String>,
}

impl TypeMapKey for SongMetadataKey {
    type Value = SongMetadata;
}

mod ytdl {
    use std::{io::ErrorKind, time::Duration};

    use songbird::input::AudioStreamError;
    use tokio::process::Command;

    const YOUTUBE_DL_COMMAND: &str = "yt-dlp";

    pub struct YtDl {
        program: &'static str,
    }

    impl YtDl {
        pub fn new() -> Self {
            Self {
                program: YOUTUBE_DL_COMMAND,
            }
        }
        pub async fn search(&self, query: &str) -> anyhow::Result<Vec<Metadata>> {
            let ytdl_args = [
                &format!("ytsearch5:'{query}'"),
                "-f",
                "ba[abr>0][vcodec=none]/best",
                "--no-playlist",
                "--print",
                "title,id,duration",
                "--flat-playlist",
            ];

            let out = Command::new(self.program)
                .args(ytdl_args)
                .output()
                .await
                .map_err(|e| {
                    AudioStreamError::Fail(if e.kind() == ErrorKind::NotFound {
                        format!("could not find executable '{}' on path", self.program).into()
                    } else {
                        Box::new(e)
                    })
                })?;

            let stdout = String::from_utf8(out.stdout)?;
            let lines: Vec<&str> = stdout.lines().collect();

            let results: Vec<Metadata> = lines
                .chunks_exact(3)
                .map(|meta| {
                    let duration: Option<Duration> = String::from(meta[2])
                        .parse::<f64>()
                        .ok()
                        .map(Duration::from_secs_f64);

                    Metadata {
                        title: String::from(meta[0]),
                        id: String::from(meta[1]),
                        duration,
                    }
                })
                .collect();

            Ok(results)
        }
    }
    pub struct Metadata {
        pub title: String,
        pub id: String,
        pub duration: Option<Duration>,
    }
}
