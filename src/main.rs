use anyhow::anyhow;
use commands::{Join, Leave, List, Pause, Play, Skip};

use service::{CreateService, Service};
use shuttle_secrets::SecretStore;
use tracing::info;

mod commands;
mod events;
mod service;
mod tools;

type ShuttleResult<T> = Result<T, shuttle_runtime::Error>;

#[shuttle_runtime::main]
async fn serenity(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttleResult<Service> {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    let service = CreateService::new(token)
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
