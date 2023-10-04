use std::sync::Arc;

use reqwest::Client as HttpClient;
use serenity::{
    async_trait,
    framework::StandardFramework,
    prelude::{GatewayIntents, TypeMapKey},
    Client as SerenityClient,
};
use songbird::SerenityInit;
use tracing::info;

use crate::{commands::Command, events::handler::Handler};


pub struct Service {
    serenity: SerenityClient,
}

pub struct CreateService {
    token: String,
    guild_id: String,
    commands: Vec<Arc<dyn Command + Sync + Send + 'static>>,
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
        self.commands.push(Arc::new(cmd));
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
        let result = self
            .serenity
            .start()
            .await
            .map_err(shuttle_runtime::CustomError::new);
        info!(?result);

        let _ = result?;

        Ok(())
    }
}

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}
