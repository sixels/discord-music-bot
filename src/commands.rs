use serenity::client::Context;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::interaction::InteractionResponseType;
use tracing::error;

/// Join your current voice channel
pub async fn join(ctx: &Context, cmd: &ApplicationCommandInteraction) {
    let guild = ctx.cache.guild(cmd.guild_id.unwrap()).unwrap();
    // let guild = cmd.guild(&ctx.cache).unwrap();
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
            respond(ctx, cmd, "You should enter a voice channel first").await;
            return;
            // return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _handler = manager.join(guild_id, connect_to).await;

    // cmd.create_interaction_response(&ctx.http, |r| r.kind(InteractionResponseType::Pong))
    //     .await
    //     .unwrap();
}

// Checks that a message successfully sent; if not, then logs why to stdout.
// fn check_msg(result: SerenityResult<Message>) {
//     if let Err(why) = result {
//         println!("Error sending message: {:?}", why);
//     }
// }

async fn respond(ctx: &Context, cmd: &ApplicationCommandInteraction, message: &str) {
    if let Err(why) = cmd
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| msg.content(message))
        })
        .await
    {
        error!("Cannot respond to slash command: {}", why);
    }
}
