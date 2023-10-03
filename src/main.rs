use anyhow::anyhow;
use commands::{Join, Leave, List, Pause, Play, Skip};

use service::{CreateService, Service};
use shuttle_secrets::SecretStore;
use tracing::info;

mod commands;
mod events;
mod service;

type ShuttleResult<T> = Result<T, shuttle_runtime::Error>;

#[shuttle_runtime::main]
async fn serenity(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttleResult<Service> {
    {
        let args = std::env::args();
        let env: std::env::Vars = std::env::vars();
        let env = env.collect::<Vec<_>>();
        info!(?args, ?env, "starting program");
    };

    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    let guild_id_var = if cfg!(debug_assertions) {
        "GUILD_ID"
    } else {
        "GUILD_ID_PROD"
    };

    let guild_id = if let Some(token) = secret_store.get(guild_id_var) {
        token
    } else {
        return Err(anyhow!("'{guild_id_var}' was not found").into());
    };

    let service = CreateService::new(token, guild_id)
        .with_command(Join)
        .with_command(Leave)
        .with_command(Play)
        .with_command(Pause)
        .with_command(Skip)
        .with_command(List)
        .create()
        .await;
    info!("Service created");

    Ok(service)
}
