use chariot_core::GLOBAL_CONFIG;

mod game;

fn main() {
    let ip_addr = format!("{}:{}", GLOBAL_CONFIG.server_address, GLOBAL_CONFIG.port);
    let game_client = game::GameClient::new(ip_addr);
    game_client.ping();
    // temporary code to keep the client on long enough to flush the initial TCP traffic
    loop {}
}
