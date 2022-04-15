use chariot_core::GLOBAL_CONFIG;

use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use tungstenite::accept;

mod game;
mod physics;

fn main() {
    let mut audience_input = Arc::new(Mutex::new(Vec::<String>::new()));
    let mut server_messages = Arc::new(Mutex::new(Vec::<String>::new()));

    // kick off the game loop
    ws(Arc::clone(&audience_input), Arc::clone(&server_messages));
    let ip_addr = format!("127.0.0.1:{}", GLOBAL_CONFIG.port);
    game::GameServer::new(
        ip_addr,
        Arc::clone(&audience_input),
        Arc::clone(&server_messages),
    )
    .start_loop();
}

fn ws(audience_input: Arc<Mutex<Vec<String>>>, server_messages: Arc<Mutex<Vec<String>>>) {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let msg = websocket.read_message().unwrap();

                // We do not want to send back ping/pong messages.
                if msg.is_binary() || msg.is_text() {
                    // when we get a message here, we want to somehow communicate this with the regular game server
                    audience_input.lock().unwrap().push(
                        msg.into_text()
                            .expect("msg should have been of type string"),
                    );
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
