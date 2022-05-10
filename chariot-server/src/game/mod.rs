use std::collections::HashMap;
use std::net::TcpListener;
use std::thread::{self};
use std::time::{Duration, Instant};

use chariot_core::networking::ws::WSAudienceBoundMessage;
use chariot_core::networking::Uuid;
use chariot_core::networking::{
    ClientBoundPacket, ClientConnection, ServerBoundPacket, WebSocketConnection,
};
use chariot_core::player_inputs::InputEvent;
use chariot_core::GLOBAL_CONFIG;

use crate::checkpoints::{FinishLine, MajorCheckpoint, MinorCheckpoint};
use crate::physics::player_entity::PlayerEntity;
use crate::physics::trigger_entity::TriggerEntity;

use self::phase::*;

mod phase;
mod voting;

pub struct GameServer {
    listener: TcpListener,
    ws_server: TcpListener,
    connections: Vec<ClientConnection>,
    ws_connections: HashMap<Uuid, WebSocketConnection>,
    game_state: ServerGameState,
    map: Option<Map>,
}

pub struct ServerGameState {
    phase: GamePhase,

    // gets its own slot because it persists across several phases; is awkward to behave identically in all
    players: [PlayerEntity; 4],

    waiting_for_player_ready_state: WaitingForPlayerReadyState,
    counting_down_to_game_start_state: CountingDownToGameStartState,
    playing_before_voting_state: PlayingBeforeVotingState,
    playing_with_voting_state: PlayingWithVotingState,
    all_players_done_state: AllPlayersDoneState,
}

impl GameServer {
    pub fn new(ip_addr: String) -> GameServer {
        // start the TCP listening service
        let listener =
            TcpListener::bind(&ip_addr).expect("could not bind to configured server address");
        println!("game server now listening on {}", ip_addr);
        let ws_server = TcpListener::bind(GLOBAL_CONFIG.ws_server_port.clone())
            .expect("could not bind to ws server");

        GameServer {
            listener,
            ws_server,
            connections: Vec::new(),
            ws_connections: HashMap::new(),
            game_state: get_starting_server_state(),
            map: None,
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
                .for_each(|con| con.fetch_incoming_packets());

            // poll for ws input events
            self.ws_connections
                .iter_mut()
                .for_each(|(_, con)| con.fetch_incoming_packets());

            self.process_incoming_packets();
            self.process_ws_packets();
            self.check_audience_voting();
            self.simulate_game();

            self.sync_state();

            // empty outgoing packet queue and send to clients
            self.connections
                .iter_mut()
                .for_each(|con| con.sync_outgoing());

            self.ws_connections
                .iter_mut()
                .for_each(|(_, con)| con.sync_outgoing());

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
                    ServerBoundPacket::ChairSelectAndReady(chair_name) => {
                        self.game_state
                            .waiting_for_player_ready_state
                            .new_players_joined
                            .push(i);
                    }
                    ServerBoundPacket::InputToggle(event) => match event {
                        InputEvent::Engine(status) => {
                            self.game_state.players[i].player_inputs.engine_status = status;
                        }
                        InputEvent::Rotation(status) => {
                            self.game_state.players[i].player_inputs.rotation_status = status;
                        }
                    },
                }
            }
        }
    }

    // sends a message to all connected web clients
    fn broadcast_ws(
        ws_connections: &mut HashMap<Uuid, WebSocketConnection>,
        message: WSAudienceBoundMessage,
    ) {
        ws_connections.iter_mut().for_each(|(_, con)| {
            con.push_outgoing_messge(message.clone());
        });
    }

    // update game state
    fn simulate_game(&mut self) {
        let now = Instant::now();

        match self.game_state.phase {
            GamePhase::WaitingForPlayerReady => {
                // Add any new players
                let state = &mut self.game_state.waiting_for_player_ready_state;
                while let Some(index) = state.new_players_joined.pop() {
                    state.players_ready[index] = true;
                    self.connections[index]
                        .push_outgoing(ClientBoundPacket::PlayerNumber(index as u8));
                }
            }
            GamePhase::CountingDownToGameStart => todo!(),
            GamePhase::PlayingBeforeVoting | GamePhase::PlayingWithVoting => {
                // earlier_time.duration_since(later_time) will return 0; filter out those for which the expiration time is earlier than the current time
                for player in &mut self.game_state.players {
                    player
                        .physics_changes
                        .retain(|change| !change.expiration_time.duration_since(now).is_zero());
                    player.set_bounding_box_dimensions();
                    player.set_upward_direction_from_bounding_box();
                }

                let others = |this_index: usize| -> Vec<&PlayerEntity> {
                    self.game_state
                        .players
                        .iter()
                        .enumerate()
                        .filter(|(other_index, _)| *other_index != this_index)
                        .map(|(_, player_entity)| player_entity)
                        .collect()
                };

                let triggers: &mut Vec<Box<&dyn TriggerEntity>> = &mut Vec::new();
                if let Some(map) = &self.map {
                    for checkpoint in map.checkpoints.iter() {
                        triggers.push(Box::new(checkpoint));
                    }

                    for zone in map.major_zones.iter() {
                        triggers.push(Box::new(zone));
                    }

                    triggers.push(Box::new(&map.finish_line));
                }

                self.game_state.players = [0, 1, 2, 3].map(|n| {
                    self.game_state.players[n].do_physics_step(1.0, others(n), triggers.to_vec())
                });
            }
            GamePhase::AllPlayersDone => todo!(),
        }
    }

    // queue up sending updated game state
    fn sync_state(&mut self) {
        match self.game_state.phase {
            GamePhase::WaitingForPlayerReady => todo!(),
            GamePhase::CountingDownToGameStart => todo!(),
            GamePhase::PlayingBeforeVoting | GamePhase::PlayingWithVoting => {
                for connection in &mut self.connections {
                    let locations = [0, 1, 2, 3]
                        .map(|n| Some(self.game_state.players[n].entity_location.clone()));

                    connection.push_outgoing(ClientBoundPacket::LocationUpdate(locations));
                }
            }
            GamePhase::AllPlayersDone => todo!(),
        }
    }
}

pub struct Map {
    major_zones: Vec<MajorCheckpoint>,
    checkpoints: Vec<MinorCheckpoint>,
    finish_line: FinishLine,
}
