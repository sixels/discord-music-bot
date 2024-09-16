#[shuttle_runtime::main]
async fn shuttle_main() -> Result<MyService, shuttle_runtime::Error> {
    // print yt-dlp path
    let out = std::process::Command::new("which")
        .arg("yt-dlp")
        .output()
        .expect("failed to run which");
    println!("yt-dlp path: {}", String::from_utf8_lossy(&out.stdout));
    Ok(MyService {})
}

struct MyService {}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for MyService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        println!("Hello!");

        Ok(())
    }
}
