use serenity::client::Context;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::interaction::InteractionResponseType;
use tracing::{error, info};

/// Join your current voice channel
pub async fn join(ctx: &Context, cmd: &ApplicationCommandInteraction) {
    let guild = ctx.cache.guild(cmd.guild_id.unwrap()).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&cmd.user.id)
        .and_then(|voice_state| voice_state.channel_id);

    // ctx.
    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            // check_msg(cmd.reply(ctx, "Not in a voice channel").await);
            error!("not in a voice channel");
            respond(
                ctx,
                cmd,
                "Você deve entrar em um canal de voz antes de usar o comando",
            )
            .await;
            return;
            // return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let channel_name = connect_to.name(&ctx.cache).await;
    info!(channel_id = connect_to.0, ?channel_name, "joining channel");

    let (_handler, result) = manager.join(guild_id, connect_to).await;
    if let Err(cause) = result {
        error!(%cause, "failed to join channel");
        respond(
            ctx,
            cmd,
            "Não consegui entrar no canal, tente novamente mais tarde",
        )
        .await;
        return;
    }

    let channel_name_str = if let Some(name) = channel_name.as_deref() {
        name
    } else {
        "?"
    };
    respond(ctx, cmd, &format!("Entrou em {channel_name_str}")).await;
}

pub async fn leave(ctx: &Context, cmd: &ApplicationCommandInteraction) {
    let guild = ctx.cache.guild(cmd.guild_id.unwrap()).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if manager.get(guild_id).is_some() {
        if let Err(cause) = manager.remove(guild_id).await {
            error!(%cause, "failed to leave channel");
            respond(ctx, cmd, "Não consegui sair do canal").await;
        } else {
            respond(ctx, cmd, "Saiu do canal").await;
        }
    } else {
        respond(ctx, cmd, "Não estou em nenhum canal").await;
    }
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
