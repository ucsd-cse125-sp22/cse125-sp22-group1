use std::collections::HashMap;
use std::net::TcpListener;
use std::thread::{self};
use std::time::{Duration, Instant};

use chariot_core::networking::ws::{QuestionBody, WSAudienceBoundMessage};
use chariot_core::networking::Uuid;
use chariot_core::networking::{
    ClientBoundPacket, ClientConnection, ServerBoundPacket, WebSocketConnection,
};
use chariot_core::player_inputs::InputEvent;
use chariot_core::GLOBAL_CONFIG;

use crate::chairs::get_player_start_physics_properties;
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
            game_state: ServerGameState {
                // notable: we don't allow more than 4 players
                phase: GamePhase::WaitingForPlayerReady(WaitingForPlayerReadyState {
                    players_ready: [false, false, false, false],
                    new_players_joined: Vec::new(),
                }),
                players: [0, 1, 2, 3]
                    .map(|num| get_player_start_physics_properties(&String::from("standard"), num)),
            },
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
                        if let GamePhase::WaitingForPlayerReady(state) = &mut self.game_state.phase
                        {
                            state.new_players_joined.push((chair_name, i));
                        }
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
            con.push_outgoing_message(message.clone());
        });
    }

    // update game state
    fn simulate_game(&mut self) {
        println!("We are in the {:?} phase!", self.game_state.phase);
        let now = Instant::now();
        match &mut self.game_state.phase {
            GamePhase::WaitingForPlayerReady(state) => {
                // Add any new players
                while let Some((chair_name, index)) = state.new_players_joined.pop() {
                    state.players_ready[index] = true;
                    self.game_state.players[index] =
                        get_player_start_physics_properties(&chair_name, index as u8);
                    self.connections[index]
                        .push_outgoing(ClientBoundPacket::PlayerNumber(index as u8));
                }

                if state.players_ready.iter().all(|&x| x)
                    || GLOBAL_CONFIG.bypass_multiplayer_requirement
                {
                    let time_until_start = Duration::new(10, 0);
                    self.game_state.phase =
                        GamePhase::CountingDownToGameStart(CountingDownToGameStartState {
                            countdown_end_time: now + time_until_start,
                        });

                    for connection in &mut self.connections {
                        connection.push_outgoing(ClientBoundPacket::GameStart(time_until_start));
                    }
                }
            }

            GamePhase::CountingDownToGameStart(state) => {
                if now > state.countdown_end_time {
                    // start off with 10 seconds of vote free gameplay
                    self.game_state.phase = GamePhase::PlayingGame(PlayingGameState {
                        voting_game_state: VotingState::VoteCooldown(now + Duration::new(10, 0)),
                    })
                }
            }

            GamePhase::PlayingGame(state) => {
                for player in &mut self.game_state.players {
                    player
                        .physics_changes
                        .retain(|change| change.expiration_time > now);
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

                match &mut state.voting_game_state {
                    VotingState::WaitingForVotes(voting_state) => {
                        if voting_state.vote_close_time < now {
                            let winner = voting_state
                                .audience_votes
                                .iter()
                                .max_by(|a, b| a.1.cmp(&b.1))
                                .map(|(_key, vote)| vote)
                                .unwrap_or(&0);

                            GameServer::broadcast_ws(
                                &mut self.ws_connections,
                                WSAudienceBoundMessage::Winner(*winner),
                            );

                            state.voting_game_state = VotingState::VoteResultActive(*winner);
                        }
                    }
                    VotingState::VoteResultActive(decision) => {
                        println!("The audience has chosen {}", decision);
                    }
                    VotingState::VoteCooldown(cooldown) => {
                        if *cooldown < now {
                            let time_until_voting_enabled = Duration::new(30, 0);
                            // somehow get a random question
                            let question: QuestionBody = (
                                "Some Question".to_string(),
                                vec![
                                    "Option 1".to_string(),
                                    "Option 2".to_string(),
                                    "Option 3".to_string(),
                                    "Option 4".to_string(),
                                ],
                            );

                            self.game_state.phase = GamePhase::PlayingGame(PlayingGameState {
                                voting_game_state: VotingState::WaitingForVotes(
                                    WaitingForVotesState {
                                        audience_votes: HashMap::new(),
                                        current_question: question.clone(),
                                        vote_close_time: now + time_until_voting_enabled, // now + 30 seconds
                                    },
                                ),
                            });

                            GameServer::broadcast_ws(
                                &mut self.ws_connections,
                                WSAudienceBoundMessage::Prompt(question.clone()),
                            );
                        }
                    }
                }
            }

            GamePhase::AllPlayersDone(_) => todo!(),
        }
    }

    // queue up sending updated game state
    fn sync_state(&mut self) {
        match self.game_state.phase {
            GamePhase::WaitingForPlayerReady(_) => {}

            // These three phases have visible players
            GamePhase::CountingDownToGameStart(_) | GamePhase::PlayingGame(_) => {
                for connection in &mut self.connections {
                    let locations = [0, 1, 2, 3]
                        .map(|n| Some(self.game_state.players[n].entity_location.clone()));

                    connection.push_outgoing(ClientBoundPacket::LocationUpdate(locations));
                }
            }

            GamePhase::AllPlayersDone(_) => todo!(),
        }
    }
}

pub struct Map {
    major_zones: Vec<MajorCheckpoint>,
    checkpoints: Vec<MinorCheckpoint>,
    finish_line: FinishLine,
}
