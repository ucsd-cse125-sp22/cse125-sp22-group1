use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::{self, spawn};
use std::time::{Duration, Instant};

use chariot_core::networking::{
    ClientConnection, ClientUpdatingPacket, ServerUpdatingPacket, WebSocketConnection,
};
use chariot_core::GLOBAL_CONFIG;
use tungstenite::{accept, Message};

use crate::physics::player_entity::PlayerEntity;

pub struct GameServer {
    listener: TcpListener,
    ws_server: TcpListener,
    connections: Vec<ClientConnection>,
    ws_connections: Vec<WebSocketConnection>,
    ws_messages: Arc<Mutex<Vec<String>>>,

    game_state: ServerGameState,
}

pub struct ServerGameState {
    players: Vec<PlayerEntity>,
}

impl GameServer {
    pub fn new(ip_addr: String) -> GameServer {
        // start the TCP listening service
        let listener =
            TcpListener::bind(&ip_addr).expect("could not bind to configured server address");
        println!("game server now listening on {}", ip_addr);
        let ws_server = TcpListener::bind("127.0.0.1:9001").expect("could not bind to ws server");

        GameServer {
            listener,
            ws_server,
            connections: Vec::new(),
            ws_connections: Vec::new(),
            ws_messages: Arc::new(Mutex::new(Vec::new())),
            game_state: ServerGameState {
                players: Vec::new(),
            },
        }
    }

    // WARNING: this function never returns
    pub fn start_loop(&mut self) {
        let max_server_tick_duration = Duration::from_millis(GLOBAL_CONFIG.server_tick_ms);

        loop {
            self.block_until_minimum_connections();
            self.acquire_any_audience_connections();

            let start_time = Instant::now();

            // poll for input events and add them to the incoming packet queue
            self.connections
                .iter_mut()
                .for_each(|con| con.sync_incoming());

            // poll for ws input events
            self.ws_connections
                .iter_mut()
                .for_each(|con| con.sync_incoming());

            self.process_incoming_packets();
            self.process_ws_packets();
            self.simulate_game();
            self.sync_state();

            // empty outgoing packet queue and send to clients
            self.connections
                .iter_mut()
                .for_each(|con| con.sync_outgoing());

            self.ws_connections
                .iter_mut()
                .for_each(|con| con.sync_outgoing());

            // wait until server tick time has elapsed
            println!("server tick time: {:#?}", start_time.elapsed());
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

    // creates a websocket for any audience connections
    fn acquire_any_audience_connections(&mut self) {
        println!("acquiring audience connections");
        let mut conns = 0;
        self.ws_server
            .set_nonblocking(true)
            .expect("non blocking should be ok");
        for stream in self.ws_server.incoming() {
            match stream {
                Ok(_) => {
                    println!("we have a stream now");
                    self.ws_connections.push(WebSocketConnection::new(
                        stream.expect("stream should be valid"),
                    ));
                    conns += 1;
                    println!("acquired an audience connection!");
                }
                Err(_) => {}
            }
            break;
        }
        self.ws_server
            .set_nonblocking(false)
            .expect("non blocking should be ok");
        println!("acquired {} audience connections!", conns);
    }

    // handle every packet in received order
    fn process_incoming_packets(&mut self) {
        for (i, connection) in self.connections.iter_mut().enumerate() {
            while let Some(packet) = connection.pop_incoming() {
                match packet {
                    ServerUpdatingPacket::Ping => {
                        println!("Received a Ping packet from client #{}!", i);
                        connection.push_outgoing(ClientUpdatingPacket::Pong);
                        self.ws_connections.iter_mut().for_each(|ws| {
                            ws.push_outgoing(Message::Text(format!(
                                "broadcasting that the server got a ping packet from client #{}!",
                                i
                            )))
                        })
                    }
                }
            }
        }
    }

    // handle socket data
    fn process_ws_packets(&mut self) {
        for (i, connection) in self.ws_connections.iter_mut().enumerate() {
            while let Some(packet) = connection.pop_incoming() {
                match packet {
                    tungstenite::Message::Text(txt) => {
                        println!(
                            "got message from client #{} of type Text, it says {}",
                            i, txt
                        )
                    }
                    tungstenite::Message::Binary(_) => {
                        println!("got message from client #{} of type Binary", i)
                    }
                    tungstenite::Message::Ping(_) => {
                        println!("got message from client #{} of type Ping", i)
                    }
                    tungstenite::Message::Pong(_) => {
                        println!("got message from client #{} of type Pong", i)
                    }
                    tungstenite::Message::Close(_) => {
                        println!("got message from client #{} of type Close", i)
                    }
                }
            }
        }
        // for stream in self.ws_server.incoming() {
        //     println!("handling a stream");
        //     let ws_messages = self.ws_messages.clone();
        //     spawn(move || {
        //         let mut websocket = accept(stream.unwrap()).unwrap();
        //         loop {
        //             let msg = websocket.read_message().unwrap();

        //             // We do not want to send back ping/pong messages.
        //             if msg.is_binary() || msg.is_text() {
        //                 let msg_txt = msg.clone().into_text().expect("expected a string");
        //                 // when we get a message here, we want to somehow communicate this with the regular game server
        //                 websocket
        //                     .write_message(tungstenite::Message::Text(
        //                         "we just got a message!".to_string(),
        //                     ))
        //                     .unwrap();
        //                 websocket.write_message(msg).unwrap();
        //                 ws_messages
        //                     .lock()
        //                     .unwrap()
        //                     .push(format!("{}", msg_txt).to_string());
        //                 println!("messages so far: {}", ws_messages.lock().unwrap().len());
        //             }
        //         }
        //     });
        // }
        // println!("done processing");
    }

    // update game state
    fn simulate_game(&mut self) {
        let mut new_players = vec![];

        for player in &mut self.game_state.players {
            player.set_bounding_box_dimensions();
        }

        for (this_index, player) in self.game_state.players.iter().enumerate() {
            let others = self
                .game_state
                .players
                .iter()
                .enumerate()
                .filter(|(other_index, _)| *other_index != this_index)
                .map(|(_, player_entity)| player_entity)
                .collect();

            new_players.push(player.do_physics_step(1.0, others));
        }

        self.game_state.players = new_players;
    }

    // queue up sending updated game state
    fn sync_state(&mut self) {}
}
