use chariot_core::GLOBAL_CONFIG;

use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use tungstenite::accept;

mod game;
mod physics;

fn main() {
    // kick off the game loop
    let ip_addr = format!("127.0.0.1:{}", GLOBAL_CONFIG.port);
    game::GameServer::new(ip_addr).start_loop();
}

fn ws() {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let msg = websocket.read_message().unwrap();

                // We do not want to send back ping/pong messages.
                if msg.is_binary() || msg.is_text() {
                    // when we get a message here, we want to somehow communicate this with the regular game server
                    websocket
                        .write_message(tungstenite::Message::Text(
                            "we just got a message!".to_string(),
                        ))
                        .unwrap();
                    websocket.write_message(msg).unwrap();
                }
            }
        });
    }
}
