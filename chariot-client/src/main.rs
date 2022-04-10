use chariot_core::GLOBAL_CONFIG;

mod game;

fn main() {
    let ip_addr = format!("{}:{}", GLOBAL_CONFIG.server_address, GLOBAL_CONFIG.port);
    let mut game_client = game::GameClient::new(ip_addr);

    // temporary code until we establish an actual game loop
    game_client.ping();
    game_client.sync_outgoing();
    loop {
        game_client.sync_incoming();
        game_client.process_incoming_packets();
    }
    // end temporary code
}
