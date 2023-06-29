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
            .arg("python3-pip")
            .status()
            .expect("failed to run apt")
            .success()
        {
            panic!("failed to install dependencies")
        }

        if !std::process::Command::new("python3")
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("-U")
            .arg("yt-dlp")
            .status()
            .expect("failed to run python3")
            .success()
        {
            panic!("failed to install yt-dlp")
        }
    }
}
