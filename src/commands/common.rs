use serenity::{
    all::{ChannelId, CommandInteraction, GuildId, ResolvedOption, ResolvedValue},
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

pub async fn respond(ctx: &Context, cmd: &CommandInteraction, message: impl Into<String>) {
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
