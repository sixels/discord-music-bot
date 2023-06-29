use serenity::{
    all::{CommandInteraction, CommandOptionType},
    async_trait,
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use songbird::input::YoutubeDl;
use tracing::{error, info};

use crate::HttpKey;

use super::respond;

/// Play a song from the given URL
pub struct Play;

#[async_trait]
impl super::Command for Play {
    fn name() -> String {
        String::from("play")
    }

    fn register() -> CreateCommand {
        CreateCommand::new(Self::name())
            .description("Play a song from the given URL")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "query",
                    "A URL from a youtube video",
                )
                .required(true),
            )
    }

    async fn run(ctx: &Context, cmd: &CommandInteraction) {
        if let Some(option) = cmd.data.options.get(0) {
            if let Some(url) = option.value.as_str() {
                info!(url, "preparing to play song");
                let guild_id = ctx.cache.guild(cmd.guild_id.unwrap()).unwrap().id;

                let manager = songbird::get(ctx)
                    .await
                    .expect("Songbird Voice client placed in at initialisation.")
                    .clone();

                let handler_lock = match manager.get(guild_id) {
                    Some(handler) => handler,
                    None => {
                        // manager.join(guild_id, connect_to)

                        let channel_id = ctx
                            .cache
                            .guild(cmd.guild_id.unwrap())
                            .unwrap()
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
                        manager
                            .join(guild_id, connect_to)
                            .await
                            .expect("could not join channel")
                    }
                };

                let mut handler = handler_lock.lock().await;

                let http = {
                    let data = ctx.data.read().await;
                    data.get::<HttpKey>()
                        .cloned()
                        .expect("Guaranteed to exist in the typemap.")
                };

                let source = YoutubeDl::new(http, url.to_string());

                handler.play_only_input(source.into());
                respond(ctx, cmd, &format!("Tocando {url}")).await;
            } else {
                respond(ctx, cmd, "URL invalida").await;
            }
        } else {
            respond(ctx, cmd, "Você precisa passar uma URL para tocar").await;
        }
    }
}
