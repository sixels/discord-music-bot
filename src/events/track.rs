use std::sync::Arc;

use serenity::{
    all::{ChannelId, ReactionType},
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::Colour,
    prelude::Context,
};
use songbird::{Event, EventContext, EventHandler};
use tracing::{error, info};

lazy_static::lazy_static! {
    static ref PREV_SONG_REACTION: ReactionType = ReactionType::Unicode(String::from("⏪"));
    static ref NEXT_SONG_REACTION: ReactionType = ReactionType::Unicode(String::from("⏩"));
    static ref PLAYPAUSE_SONG_REACTION: ReactionType = ReactionType::Unicode(String::from("⏯️"));
}

pub struct PlayingSongNotifier {
    pub channel_id: ChannelId,
    pub http: Arc<Http>,
    pub context: Context,

    pub title: String,
    pub username: String,
    pub thumbnail: Option<String>,
}

#[async_trait]
impl EventHandler for PlayingSongNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            info!(
                "playing a song. there are {} items in the queue.",
                track_list.len()
            );

            let mut embed = CreateEmbed::new()
                .color(Colour::BLUE)
                .field("TOCANDO AGORA", self.title.clone(), true)
                .field("", format!("requisitado por **{}**", self.username), false);

            if let Some(thumb) = self.thumbnail.clone() {
                embed = embed.thumbnail(thumb);
            }

            match self
                .channel_id
                .send_message(
                    &self.http,
                    CreateMessage::new().add_embed(embed), // .reactions([
                                                           //     PREV_SONG_REACTION.clone(),
                                                           //     PLAYPAUSE_SONG_REACTION.clone(),
                                                           //     NEXT_SONG_REACTION.clone(),
                                                           // ]),
                )
                .await
            {
                Ok(_message) => {
                    // if let Some((_, handler)) = track_list.get(0) {
                    //     while let Some(reaction) =
                    //         message.await_reactions(&self.context).next().await
                    //     {
                    //         if reaction.emoji == PREV_SONG_REACTION.clone() {
                    //             info!("playing previous track");
                    //             break;
                    //         }
                    //         if reaction.emoji == NEXT_SONG_REACTION.clone() {
                    //             info!("playing next track");
                    //             break;
                    //         }
                    //         if reaction.emoji == PLAYPAUSE_SONG_REACTION.clone() {
                    //             info!("toggling the playing state track");
                    //             handler.add_event(Event::Track(TrackEvent::Play), action)
                    //         }
                    //     }
                    // }
                }
                Err(cause) => error!(%cause, "failed to send message"),
            }
        }

        None
    }
}
