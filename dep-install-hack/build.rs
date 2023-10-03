fn main() {
    // Install external dependency (in the shuttle container only)
    if std::env::var("HOSTNAME")
        .unwrap_or_default()
        .contains("shuttle")
    {
        if !std::process::Command::new("apt")
            .arg("install")
            .arg("-y")
            .arg("libopus-dev")
            .arg("ffmpeg")
            .arg("pipx")
            .status()
            .expect("failed to run apt")
            .success()
        {
            panic!("failed to install dependencies")
        }

        std::process::Command::new("pipx")
            .arg("ensurepath")
            .status()
            .expect("failed to run pipx");

        if !std::process::Command::new("pipx")
            .arg("install")
            .arg("yt-dlp")
            .status()
            .expect("failed to run pipx")
            .success()
        {
            panic!("failed to install yt-dlp")
        }
    }
}
