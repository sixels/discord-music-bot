use serenity::{
    all::{CommandInteraction, CommandOptionType},
    async_trait,
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use songbird::{
    input::{Compose, YoutubeDl},
    Event,
};
use tracing::info;

use crate::{
    commands::{common, join::join_channel},
    events::track::PlayingSongNotifier,
    HttpKey,
};

/// Play a song from the given URL
pub struct Play;

#[async_trait]
impl super::Command for Play {
    fn name(&self) -> String {
        String::from("play")
    }

    fn register(&self, cmd: CreateCommand) -> CreateCommand {
        cmd.description("Play a song from the given URL")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "query",
                    "A URL from a youtube video",
                )
                .required(true),
            )
    }

    async fn run(&self, ctx: &Context, cmd: &CommandInteraction) {
        let url_option = cmd
            .data
            .options
            .first()
            .and_then(|option| option.value.as_str());

        let url = match url_option {
            Some(url) => url,
            None => {
                common::respond(ctx, cmd, "Você precisa passar uma URL válida para tocar").await;
                return;
            }
        };
        info!(url, "preparing to play song");

        let guild_id = common::get_guild_id(ctx, cmd);

        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.");

        let handler_lock = match manager.get(guild_id) {
            Some(handler) => handler,
            None => match join_channel(manager, ctx, cmd).await {
                Ok((handler, _)) => handler,
                Err(e) => {
                    common::respond(ctx, cmd, &e).await;
                    return;
                }
            },
        };
        let mut handler = handler_lock.lock().await;

        let http = {
            let data = ctx.data.read().await;
            data.get::<HttpKey>()
                .cloned()
                .expect("Guaranteed to exist in the typemap.")
        };

        let mut source = YoutubeDl::new(http, url.to_string());
        let meta = source.aux_metadata().await.ok();

        let title;
        if let Some(metadata) = meta {
            title = metadata.title.unwrap_or(String::new());
        } else {
            title = url.to_string()
        }

        let song = handler.enqueue_input(source.into()).await;
        song.add_event(
            Event::Track(songbird::TrackEvent::Play),
            PlayingSongNotifier {
                channel_id: cmd.channel_id,
                http: ctx.http.clone(),
                title: title.clone(),
            },
        )
        .ok();

        common::respond(ctx, cmd, &format!("Adicionado: `{title}`")).await;
    }
}
