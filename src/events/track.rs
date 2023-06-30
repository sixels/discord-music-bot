use std::sync::Arc;

use serenity::{all::ChannelId, async_trait, http::Http};
use songbird::{Event, EventContext, EventHandler};
use tracing::info;

pub struct PlayingSongNotifier {
    pub channel_id: ChannelId,
    pub http: Arc<Http>,
    pub title: String,
}

#[async_trait]
impl EventHandler for PlayingSongNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            info!(
                "playing a song. there are {} items in the queue.",
                track_list.len()
            );
            self.channel_id
                .say(&self.http, &format!("Tocando: {}.", self.title))
                .await
                .ok();
        }

        None
    }
}
