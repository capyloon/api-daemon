use b2ghald::client::SimpleClient;
use log::info;

fn main() {
    env_logger::init();

    let mut client = SimpleClient::new().expect("Failed to connect to b2ghald");
    info!(
        "Current brightness on default screen is {}",
        client.get_screen_brightness(0)
    );
}
