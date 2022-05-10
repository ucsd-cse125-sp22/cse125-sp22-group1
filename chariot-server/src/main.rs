use chariot_core::GLOBAL_CONFIG;

mod chairs;
mod game;
mod physics;

fn main() {
    // kick off the game loop
    let ip_addr = format!("0.0.0.0:{}", GLOBAL_CONFIG.port);
    game::GameServer::new(ip_addr).start_loop();
}
