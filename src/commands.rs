mod join;
mod leave;
mod pause;
mod play;

use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::client::Context;
use serenity::{all::CommandInteraction, builder::CreateCommand};
use tracing::error;

pub use self::{join::Join, leave::Leave, play::Play};

#[serenity::async_trait]
pub trait Command {
    fn name() -> String;
    fn register() -> CreateCommand;

    #[allow(unused_variables)]
    async fn run(ctx: &Context, cmd: &CommandInteraction) {}
}

async fn respond(ctx: &Context, cmd: &CommandInteraction, message: &str) {
    if let Err(cause) = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(message),
            ),
        )
        .await
    {
        error!(%cause, "failed to respond to slash command");
    }
}
