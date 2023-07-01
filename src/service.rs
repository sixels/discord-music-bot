use reqwest::Client as HttpClient;
use serenity::{
    async_trait, framework::StandardFramework, prelude::GatewayIntents, Client as SerenityClient,
};
use songbird::SerenityInit;

use crate::{commands::Command, Handler, HttpKey};

pub struct Service {
    serenity: SerenityClient,
}

pub struct CreateService {
    token: String,
    guild_id: String,
    commands: Vec<Box<dyn Command + Sync + Send + 'static>>,
}

impl CreateService {
    pub fn new(token: String, guild_id: String) -> Self {
        Self {
            token,
            guild_id,
            commands: Vec::new(),
        }
    }

    pub fn with_command<C: Command + Sync + Send + 'static>(mut self, cmd: C) -> Self {
        self.commands.push(Box::new(cmd));
        self
    }

    pub async fn create(self) -> Service {
        let framework: StandardFramework = StandardFramework::new();
        let intents = GatewayIntents::non_privileged();

        let handler = Handler {
            guild_id: self.guild_id,
            commands: self.commands,
        };
        let client = SerenityClient::builder(&self.token, intents)
            .event_handler(handler)
            .framework(framework)
            .register_songbird()
            .type_map_insert::<HttpKey>(HttpClient::new())
            .await
            .expect("Err creating client");

        Service { serenity: client }
    }
}

#[async_trait]
impl shuttle_runtime::Service for Service {
    async fn bind(mut self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        self.serenity
            .start()
            .await
            .map_err(shuttle_runtime::CustomError::new)?;
        Ok(())
    }
}
