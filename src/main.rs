use anyhow::anyhow;
use commands::{join, leave, list, pause, play};
// skip};

use service::CreateService;
use shuttle_runtime::SecretStore;
use tracing::info;

mod commands;
mod events;
mod service;
mod tools;

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // print yt-dlp path
    let out = std::process::Command::new("which")
        .arg("yt-dlp")
        .output()
        .expect("failed to run which");
    println!("yt-dlp path: {}", String::from_utf8_lossy(&out.stdout));

    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secrets.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // add /usr/bin to PATH
    unsafe {
        let path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("/bin:/usr/bin:/usr/local/bin:{}", path));
    }

    let service = CreateService::new(token)
        .with_command(join())
        .with_command(leave())
        .with_command(play())
        .with_command(pause())
        // .with_command(skip())
        .with_command(list())
        .build()
        .await;
    info!("Service created");

    // todo!()
    Ok(service)
}
