use poise::Command;
use reqwest::Client as HttpClient;
use serenity::{
    prelude::{GatewayIntents, TypeMapKey},
    Client as SerenityClient,
};
use shuttle_serenity::SerenityService;
use songbird::SerenityInit;

use crate::commands;

pub struct CreateService {
    token: String,
    commands: Vec<Command<(), commands::Error>>,
}

impl CreateService {
    pub fn new(token: String) -> Self {
        Self {
            token,
            commands: Vec::new(),
        }
    }

    pub fn with_command(mut self, cmd: Command<(), commands::Error>) -> Self {
        self.commands.push(cmd);
        self
    }

    pub async fn build(self) -> SerenityService {
        let framework: poise::Framework<(), anyhow::Error> = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: self.commands,
                ..Default::default()
            })
            .setup(|ctx, _, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(())
                })
            })
            .build();

        let intents = GatewayIntents::non_privileged();

        let client = SerenityClient::builder(&self.token, intents)
            .framework(framework)
            .register_songbird()
            .type_map_insert::<HttpKey>(HttpClient::new())
            .await
            .expect("Err creating client");

        client.into()
    }
}

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}
