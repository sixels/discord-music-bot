use serenity::{
    all::{
        ChannelId, CommandInteraction, ComponentInteraction, GuildId, ResolvedOption, ResolvedValue,
    },
    async_trait,
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    client::Context,
};
use tracing::error;

pub fn get_guild_id(ctx: &Context, cmd: &CommandInteraction) -> GuildId {
    let guild = ctx
        .cache
        .guild(cmd.guild_id.expect("Not in a guild"))
        .unwrap();

    guild.id
}

pub fn get_guild_and_channel(
    ctx: &Context,
    cmd: &CommandInteraction,
) -> (GuildId, Option<ChannelId>) {
    let guild = ctx
        .cache
        .guild(cmd.guild_id.expect("Not in a guild"))
        .unwrap();

    let channel_id = guild
        .voice_states
        .get(&cmd.user.id)
        .and_then(|voice_state| voice_state.channel_id);

    (guild.id, channel_id)
}

pub fn get_option<'r, 'o>(
    opts: &'r [ResolvedOption<'o>],
    name: &'_ str,
) -> Option<&'r ResolvedValue<'o>> {
    opts.iter()
        .find(|option| option.name == name)
        .map(|opt| &opt.value)
}

#[async_trait]
pub trait CanRespond {
    async fn respond_to(&self, ctx: &Context, message: String);
}

#[async_trait]
impl CanRespond for CommandInteraction {
    async fn respond_to(&self, ctx: &Context, message: String) {
        if let Err(cause) = self
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
}
#[async_trait]
impl CanRespond for &CommandInteraction {
    async fn respond_to(&self, ctx: &Context, message: String) {
        if let Err(cause) = self
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
}

#[async_trait]
impl CanRespond for ComponentInteraction {
    async fn respond_to(&self, ctx: &Context, message: String) {
        if let Err(cause) = self
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
}

pub async fn respond<R: CanRespond>(ctx: &Context, r: &R, message: impl Into<String>) {
    r.respond_to(ctx, message.into()).await;
}
