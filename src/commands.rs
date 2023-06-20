mod join;
mod leave;
mod pause;
mod play;

use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::interaction::InteractionResponseType;
use tracing::error;

pub use self::{join::Join, leave::Leave, play::Play};

#[serenity::async_trait]
pub trait Command {
    fn name() -> String;
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;

    #[allow(unused_variables)]
    async fn run(ctx: &Context, cmd: &ApplicationCommandInteraction) {}
}

pub fn register_command<C: Command>(commands: &mut serenity::builder::CreateApplicationCommands) {
    commands.create_application_command(C::register);
}

async fn respond(ctx: &Context, cmd: &ApplicationCommandInteraction, message: &str) {
    if let Err(cause) = cmd
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| msg.content(message))
        })
        .await
    {
        error!(%cause, "failed to respond to slash command");
    }
}
