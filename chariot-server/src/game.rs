use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use chariot_core::networking::{ClientConnection, ClientUpdatingPacket, ServerUpdatingPacket};
use chariot_core::GLOBAL_CONFIG;

pub struct GameServer {
    listener: TcpListener,
    connections: Vec<ClientConnection>,
    audience_input: Arc<Mutex<Vec<String>>>,
    server_messages: Arc<Mutex<Vec<String>>>,
}

impl GameServer {
    pub fn new(
        ip_addr: String,
        audience_input: Arc<Mutex<Vec<String>>>,
        server_messages: Arc<Mutex<Vec<String>>>,
    ) -> GameServer {
        // start the TCP listening service
        let listener =
            TcpListener::bind(&ip_addr).expect("could not bind to configured server address");
        println!("game server now listening on {}", ip_addr);
        GameServer {
            listener,
            connections: Vec::new(),
            audience_input,
            server_messages,
        }
    }

    // WARNING: this function never returns
    pub fn start_loop(&mut self) {
        let max_server_tick_duration = Duration::from_millis(GLOBAL_CONFIG.server_tick_ms);

        loop {
            self.block_until_minimum_connections();

            let start_time = Instant::now();

            // poll for input events and add them to the incoming packet queue
            self.connections
                .iter_mut()
                .for_each(|con| con.sync_incoming());

            // poll for input events from websocket
            let mut audience_input = self.audience_input.lock().unwrap();

            for message in audience_input.iter() {
                println!("got message {}", message);
            }

            self.process_incoming_packets();
            self.simulate_game();
            self.sync_state();

            // empty outgoing packet queue and send to clients
            self.connections
                .iter_mut()
                .for_each(|con| con.sync_outgoing());

            // wait until server tick time has elapsed
            let remaining_tick_duration = max_server_tick_duration
                .checked_sub(start_time.elapsed())
                .expect("server tick took longer than configured length");
            thread::sleep(remaining_tick_duration);
        }
    }

    // blocks the primary loop if we don't have the minimum players
    fn block_until_minimum_connections(&mut self) {
        while self.connections.len() < GLOBAL_CONFIG.player_amount {
            match self.listener.accept() {
                Ok((socket, addr)) => {
                    println!("new connection from {}", addr.ip().to_string());
                    self.connections.push(ClientConnection::new(socket));
                }
                Err(e) => println!("couldn't get connecting client info {:?}", e),
            }
        }
    }

    // handle every packet in received order
    fn process_incoming_packets(&mut self) {
        for (i, connection) in self.connections.iter_mut().enumerate() {
            while let Some(packet) = connection.pop_incoming() {
                match packet {
                    ServerUpdatingPacket::Ping => {
                        println!("Received a Ping packet from client #{}!", i);
                        connection.push_outgoing(ClientUpdatingPacket::Pong);
                        self.server_messages
                            .lock()
                            .unwrap()
                            .push("PING".to_string());
                    }
                }
            }
        }
    }

    // update game state
    fn simulate_game(&mut self) {}

    // queue up sending updated game state
    fn sync_state(&mut self) {}
}
