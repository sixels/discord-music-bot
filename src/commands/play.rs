use std::time::Duration;

use serenity::{
    builder::{CreateButton, CreateEmbed, CreateInteractionResponseFollowup},
    model::Colour,
    prelude::TypeMapKey,
};
use songbird::{
    input::{Compose, YoutubeDl},
    Event,
};
use tracing::{error, info};

use crate::{
    commands::join::join_channel,
    events::track::PlayingSongNotifier,
    service::HttpKey,
    tools::piped::{PipedClient, PipedError},
};

use super::{Context, Error};

/// Toca uma música no canal de voz atual
#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "URL ou nome da música a ser tocada"] song: String,
) -> Result<(), Error> {
    println!("play command");
    let guild_id = ctx.guild_id().expect("guild id not found");
    let guild = guild_id.to_guild_cached(&ctx.cache()).unwrap().clone();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => match join_channel(manager, &ctx, &guild).await {
            Ok((handler, _)) => handler,
            Err(cause) => {
                error!(%cause, "failed to join channel");
                ctx.reply(cause.to_string()).await?;
                return Ok(());
            }
        },
    };

    ctx.defer_ephemeral().await?;

    let Ok(query) = QueryKind::try_from(song.as_str()) else {
        ctx.reply("URL ou nome da música inválidos").await?;
        return Ok(());
    };

    info!(?query, "searching for song");

    let http = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<HttpKey>()
            .expect("http client not found")
            .clone()
    };

    let mut source = match query {
        QueryKind::Url(url) => YoutubeDl::new_ytdl_like("/usr/bin/yt-dlp", http, url.to_string()),
        QueryKind::Search(search) => {
            let results = match PipedClient::new(&http).search_songs(search).await {
                Ok(results) => results,
                Err(err) => {
                    match err {
                        PipedError::Request => {
                            ctx.reply("Não consegui pesquisar nenhuma música").await?;
                        }
                        PipedError::Unknown => {
                            ctx.reply("Deu ruim :sob:").await?;
                        }
                    };
                    return Ok(());
                }
            };

            info!("found {} results", results.items.len());

            let results_size = results.items.len().min(5);

            let result_formats = results
                .items
                .iter()
                .take(results_size)
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
                    .field("Resultados", result_formats.join("\n"), false)
                    .colour(Colour::RED),
            );

            for (i, result) in results.items.iter().take(results_size).enumerate() {
                followup = followup
                    .button(CreateButton::new(result.url.clone()).label((i + 1).to_string()))
            }

            info!("creating follow up message");
            let poise::Context::Application(ref app_ctx) = ctx else {
                return Ok(());
            };

            let Ok(message) = app_ctx
                .interaction
                .create_followup(&ctx.http(), followup)
                .await
            else {
                ctx.reply("Deu ruim :sob:").await?;
                return Ok(());
            };

            let Some(response) = message.await_component_interaction(&ctx).await else {
                return Ok(());
            };

            let video_url = &response.data.custom_id;
            info!("user selected {}", video_url);

            let video_uri = format!("https://www.youtube.com/{video_url}");

            if let Err(cause) = app_ctx
                .interaction
                .delete_followup(&ctx.http(), message.id)
                .await
            {
                error!(%cause, "failed to delete response")
            }

            YoutubeDl::new(http, video_uri)
        }
    };

    // let _ = source.create_async().await.unwrap();

    let meta = source.aux_metadata().await.ok();

    dbg!(&meta);

    let requester = ctx
        .author()
        .global_name
        .clone()
        .unwrap_or(ctx.author().name.clone());

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

    ctx.send(
        poise::CreateReply::default()
            .content(format!("Adicionando ||{}|| à fila", song_meta.title))
            .ephemeral(false),
    )
    .await?;

    let mut handler = handler_lock.lock().await;

    let song = handler.enqueue_input(source.into()).await;

    if let Err(cause) = song.add_event(
        Event::Track(songbird::TrackEvent::Play),
        PlayingSongNotifier {
            channel_id: ctx.channel_id(),
            http: ctx.serenity_context().http.clone(),
            context: ctx.serenity_context().clone(),
            title: song_meta.title.clone(),
            username: ctx
                .author()
                .global_name
                .clone()
                .unwrap_or(ctx.author().name.clone()),
            thumbnail: song_meta.thumbnail.clone(),
        },
    ) {
        error!(%cause, "failed to create song event")
    }

    let mut typemap = song.typemap().write().await;
    typemap.insert::<SongMetadataKey>(song_meta);

    Ok(())
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

impl<'s> TryFrom<&'s str> for QueryKind<'s> {
    type Error = ();
    fn try_from(value: &'s str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(());
        }
        if value.starts_with("http://")
            || value.starts_with("https://")
            || value.starts_with("www.")
        {
            Ok(QueryKind::Url(value))
        } else {
            Ok(QueryKind::Search(value))
        }
    }
}

pub struct SongMetadataKey;

#[allow(dead_code)]
pub struct SongMetadata {
    pub title: String,
    pub duration: Duration,
    pub user: String,
    pub thumbnail: Option<String>,
}

impl TypeMapKey for SongMetadataKey {
    type Value = SongMetadata;
}
