use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use chariot_core::networking::{ClientConnection, ClientUpdatingPacket, ServerUpdatingPacket};
use chariot_core::player_inputs::InputEvent;
use chariot_core::GLOBAL_CONFIG;

use crate::physics::player_entity::PlayerEntity;

pub struct GameServer {
    listener: TcpListener,
    connections: Vec<ClientConnection>,

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
        GameServer {
            listener,
            connections: Vec::new(),
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

            let start_time = Instant::now();

            // poll for input events and add them to the incoming packet queue
            self.connections
                .iter_mut()
                .for_each(|con| con.sync_incoming());

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
                    }
                    ServerUpdatingPacket::InputToggle(event, enable) => match event {
                        InputEvent::Engine(status) => {
                            if !enable {
                                // self.players[figureOutWhoThisIs()].player_inputs.engine_status = EngineStatus::Neutral;
                                println!("Player is not moving anymore!");
                            } else {
                                // self.players[figureOutWhoThisIs()].player_inputs.engine_status = status;
                                println!("Player is moving: {:?}", status);
                            }
                        }
                        InputEvent::Rotation(status) => {
                            if !enable {
                                // self.players[figureOutWhoThisIs()].player_inputs.rotation_status = RotationStatus::NotInSpin;
                                println!("Player is not turning anymore!");
                            } else {
                                // self.players[figureOutWhoThisIs()].player_inputs.rotation_status = status;
                                println!("Player is turning: {:?}", status);
                            }
                        }
                    },
                }
            }
        }
    }

    // update game state
    fn simulate_game(&mut self) {
        let mut new_players = vec![];

        let now = Instant::now();

        // earlier_time.duration_since(later_time) will return 0; filter out those for which the expiration time is earlier than the current time
        for player in &mut self.game_state.players {
            player
                .physics_changes
                .retain(|change| !change.expiration_time.duration_since(now).is_zero());
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
