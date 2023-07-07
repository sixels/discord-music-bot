mod common;
mod join;
mod leave;
mod list;
mod pause;
mod play;
mod skip;

use serenity::client::Context;
use serenity::{all::CommandInteraction, builder::CreateCommand};

pub use self::{join::Join, leave::Leave, list::List, pause::Pause, play::Play, skip::Skip};

#[serenity::async_trait]
pub trait Command {
    fn name(&self) -> String;
    fn register(&self, cmd: CreateCommand) -> CreateCommand;

    #[allow(unused_variables)]
    async fn run(&self, ctx: &Context, cmd: &CommandInteraction) {}
}
