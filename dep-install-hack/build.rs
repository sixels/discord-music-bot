fn main() {
    // Install external dependency (in the shuttle container only)
    if !std::env::var("SHUTTLE").is_ok() {
        return;
    }

    if !std::process::Command::new("apt")
        .arg("update")
        .status()
        .expect("failed to run apt")
        .success()
    {
        panic!("failed to update apt")
    }
    if !std::process::Command::new("apt")
        .arg("install")
        .arg("-y")
        .arg("libopus-dev")
        .arg("ffmpeg")
        .arg("curl")
        .status()
        .expect("failed to run apt")
        .success()
    {
        panic!("failed to install dependencies")
    }

    // Download yt-dlp
    let yt_dlp_url = "https://github.com/yt-dlp/yt-dlp/releases/download/2024.08.06/yt-dlp_linux";
    if !std::process::Command::new("curl")
        .arg("-L")
        .arg("-o")
        .arg("/usr/bin/yt-dlp")
        .arg(yt_dlp_url)
        .status()
        .expect("failed to download yt-dlp")
        .success()
    {
        panic!("failed to download yt-dlp")
    }
    if !std::process::Command::new("chmod")
        .arg("+x")
        .arg("/usr/bin/yt-dlp")
        .status()
        .expect("failed to make yt-dlp executable")
        .success()
    {
        panic!("failed to make yt-dlp executable")
    }
}
