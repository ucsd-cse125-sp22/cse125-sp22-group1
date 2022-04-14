use chariot_core::GLOBAL_CONFIG;
use std::thread;

mod game;
mod physics;
mod web_sockets;

fn main() {
    // kick off the game loop
    let ip_addr = format!("127.0.0.1:{}", GLOBAL_CONFIG.port);
    let mut server = game::GameServer::new(ip_addr);

    let game_loop = thread::spawn(move || server.start_loop());

    let ws_loop = thread::spawn(|| web_sockets::start_ws());

    game_loop
        .join()
        .expect("should have been able to join game loop thread");

    ws_loop
        .join()
        .expect("should have been able to join web socket thread");
}
