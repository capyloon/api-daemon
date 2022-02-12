use log::debug;
use wspty::start_server;

#[tokio::main]
async fn main() {
    env_logger::init();
    let _ = start_server()
        .await
        .map_err(|e| debug!("wspty server exit with error: {:?}", e));
}
