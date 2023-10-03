use std::sync::Arc;

use serenity::{
    all::{GuildId, Interaction, Ready},
    async_trait,
    builder::CreateCommand,
    gateway::ActivityData,
    prelude::*,
};
use tracing::{error, info};

use crate::commands::Command;

pub struct Handler {
    pub guild_id: String,
    pub commands: Vec<Arc<dyn Command + Send + Sync + 'static>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(cmd) = interaction {
            let command_name = cmd.data.name.as_str();
            let command = self
                .commands
                .iter()
                .find(|c| c.name() == command_name)
                .cloned();
            // let command = command.cloned();

            if let Some(command) = command {
                // tokio::spawn(async move {
                info!("handling {} command", command.name());
                command.run(ctx, cmd).await;
                // });
            } else {
                error!("invalid command passed: {}", command_name)
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let activity = ActivityData::playing("o popozão no chão");
        ctx.set_activity(Some(activity));

        let guild_id = GuildId(self.guild_id.parse().unwrap());

        // if let Ok(cmds) = guild_id.get_commands(&ctx.http).await {
        //     info!("deleting old commands");
        //     for cmd in cmds {
        //         guild_id.delete_command(&ctx.http, cmd.id).await.ok();
        //     }
        // }

        if let Err(cause) = guild_id
            .set_commands(
                &ctx.http,
                self.commands
                    .iter()
                    .map(|c| c.register(CreateCommand::new(c.name())))
                    .collect(),
            )
            .await
        {
            panic!("failed to register commands: {cause}")
        }
    }
}
