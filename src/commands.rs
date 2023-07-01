mod common;
mod join;
mod leave;
mod pause;
mod play;

use serenity::client::Context;
use serenity::{all::CommandInteraction, builder::CreateCommand};

pub use self::{join::Join, leave::Leave, play::Play};

#[serenity::async_trait]
pub trait Command {
    fn name(&self) -> String;
    fn register(&self, cmd: CreateCommand) -> CreateCommand;

    #[allow(unused_variables)]
    async fn run(&self, ctx: &Context, cmd: &CommandInteraction) {}
}
