use std::sync::Arc;

use serenity::{
    all::{Guild, GuildId, Interaction, Ready},
    async_trait,
    builder::CreateCommand,
    gateway::ActivityData,
    prelude::*,
};
use tracing::{error, info};

use crate::commands::Command;

pub struct Handler {
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
                info!("handling command: {}", command.name());
                command.run(ctx, cmd).await;
                // });
            } else {
                error!("invalid command: {}", command_name)
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let activity = ActivityData::playing("o popozão no chão");
        ctx.set_activity(Some(activity));

        for guild_id in ctx.cache.guilds().iter().copied() {
            info!("registering commands on guild {guild_id}");
            if let Err(err) = register_commands(&ctx, guild_id, &self.commands).await {
                error!("{err:?}")
            }
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: Option<bool>) {
        if let Err(err) = register_commands(&ctx, guild.id, &self.commands).await {
            error!("{err:?}")
        }
    }
}

async fn register_commands(
    ctx: &Context,
    guild_id: GuildId,
    commands: &[Arc<dyn Command + Send + Sync + 'static>],
) -> anyhow::Result<()> {
    if let Err(cause) = guild_id
        .set_commands(
            &ctx.http,
            commands
                .iter()
                .map(|c| c.register(CreateCommand::new(c.name())))
                .collect(),
        )
        .await
    {
        return Err(anyhow::anyhow!(
            "failed to register command on guild {guild_id}: {cause}"
        ));
    }

    Ok(())
}
