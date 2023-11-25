use std::sync::Arc;

use serenity::{
    all::ChannelId,
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::Colour,
    prelude::Context,
};
use songbird::{Event, EventContext, EventHandler};
use tracing::{error, info};

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

            if let Err(cause) = self
                .channel_id
                .send_message(&self.http, CreateMessage::new().add_embed(embed))
                .await
            {
                error!(%cause, "failed to send message")
            }
        }

        None
    }
}
