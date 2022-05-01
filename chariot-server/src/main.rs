use chariot_core::GLOBAL_CONFIG;

mod chairs;
mod checkpoints;
mod game;
mod map;
mod physics;

fn main() {
    // kick off the game loop
    let ip_addr = format!("127.0.0.1:{}", GLOBAL_CONFIG.port);
    game::GameServer::new(ip_addr).start_loop();
}
