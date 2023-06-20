use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::Context,
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
    },
};
use tracing::{error, info};

use super::respond;

/// Play a song from the given URL
pub struct Play;

#[async_trait]
impl super::Command for Play {
    fn name() -> String {
        String::from("play")
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name(Self::name())
            .description("Play a song from the given URL")
            .create_option(|option| {
                option
                    .name("url")
                    .description("A URL from a youtube video")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }

    async fn run(ctx: &Context, cmd: &ApplicationCommandInteraction) {
        if let Some(option) = cmd.data.options.get(0) {
            if let CommandDataOptionValue::String(url) =
                option.resolved.as_ref().expect("Invalid option")
            {
                info!(url, "preparing to play song");
                let guild = ctx.cache.guild(cmd.guild_id.unwrap()).unwrap();
                let guild_id = guild.id;

                let manager = songbird::get(ctx)
                    .await
                    .expect("Songbird Voice client placed in at initialisation.")
                    .clone();

                let handler_lock = match manager.get(guild_id) {
                    Some(handler) => handler,
                    None => {
                        // manager.join(guild_id, connect_to)

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
                            }
                        };
                        let (h, _) = manager.join(guild_id, connect_to).await;
                        h
                    }
                };

                let mut handler = handler_lock.lock().await;

                let source = match songbird::ytdl(&url).await {
                    Ok(source) => source,
                    Err(cause) => {
                        error!(%cause, "failed to create the source");
                        respond(ctx, cmd, "Não consegui tocar essa").await;
                        return;
                    }
                };

                handler.play_only_source(source);
                respond(ctx, cmd, &format!("Tocando {url}")).await;
            } else {
                respond(ctx, cmd, "URL invalida").await;
            }
        } else {
            respond(ctx, cmd, "Você precisa passar uma URL para tocar").await;
        }
    }
}
