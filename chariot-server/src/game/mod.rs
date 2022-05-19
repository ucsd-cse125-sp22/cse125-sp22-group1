use std::collections::HashMap;
use std::net::TcpListener;
use std::thread::{self};
use std::time::{Duration, Instant};

use chariot_core::player::choices::PlayerChoices;
use chariot_core::player::lap_info::Placement;
use chariot_core::player::{
    lap_info::LapInformation,
    physics_changes::{PhysicsChange, PhysicsChangeType},
    player_inputs::InputEvent,
    PlayerID,
};
use glam::DVec3;

use chariot_core::entity_location::EntityLocation;
use chariot_core::networking::ws::WSAudienceBoundMessage;
use chariot_core::networking::Uuid;
use chariot_core::networking::{
    ClientBoundPacket, ClientConnection, ServerBoundPacket, WebSocketConnection,
};
use chariot_core::questions::{QuestionData, QUESTIONS};
use chariot_core::GLOBAL_CONFIG;

use crate::chairs::get_player_start_physics_properties;
use crate::physics::player_entity::PlayerEntity;
use crate::progress::get_player_placement_array;

use self::map::Map;
use self::phase::*;

mod map;
mod phase;
pub mod powerup;
mod voting;

pub struct GameServer {
    listener: TcpListener,
    ws_server: TcpListener,
    connections: Vec<ClientConnection>,
    ws_connections: HashMap<Uuid, WebSocketConnection>,
    game_state: ServerGameState,
}

pub struct ServerGameState {
    phase: GamePhase,

    // gets its own slot because it persists across several phases; is awkward to behave identically in all
    players: [PlayerEntity; 4],

    map: Option<Map>,
}

impl GameServer {
    pub fn new(ip_addr: String) -> GameServer {
        // start the TCP listening service
        let listener =
            TcpListener::bind(&ip_addr).expect("could not bind to configured server address");
        listener
            .set_nonblocking(true)
            .expect("Couldn't set the listener to be non-blocking!");
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
                phase: GamePhase::ConnectingAndChoosingSettings {
                    force_start: false,
                    player_choices: Default::default(),
                },
                players: [0, 1, 2, 3]
                    .map(|num| get_player_start_physics_properties(&String::from("standard"), num)),
                map: None,
            },
        }
    }

    // WARNING: this function never returns
    pub fn start_loop(&mut self) {
        let max_server_tick_duration = Duration::from_millis(GLOBAL_CONFIG.server_tick_ms);

        loop {
            // self.block_until_minimum_connections();
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
            if let Some(remaining_tick_duration) =
                max_server_tick_duration.checked_sub(start_time.elapsed())
            {
                thread::sleep(remaining_tick_duration);
            } else {
                match self.game_state.phase {
                    GamePhase::ConnectingAndChoosingSettings { .. }
                    | GamePhase::WaitingForPlayerLoad { .. } => println!("Tick took longer than configured length, but we don't care because we are still loading"),
                    _ => panic!("server tick took longer than configured length"),
                }
            }
        }
    }

    // handle every packet in received order
    fn process_incoming_packets(&mut self) {
        let mut need_to_broadcast: Vec<ClientBoundPacket> = vec![];
        for (player_num, connection) in self.connections.iter_mut().enumerate() {
            while let Some(packet) = connection.pop_incoming() {
                match packet {
                    ServerBoundPacket::ChairSelect(new_chair) => match &mut self.game_state.phase {
                        GamePhase::ConnectingAndChoosingSettings { player_choices, .. } => {
                            if let Some(PlayerChoices { chair, .. }) =
                                &mut player_choices[player_num]
                            {
                                println!(
                                    "Setting player #{}'s chair to {}!",
                                    player_num,
                                    new_chair.clone()
                                );
                                *chair = new_chair.clone();
                                need_to_broadcast.push(ClientBoundPacket::PlayerChairChoice(
                                    player_num, new_chair,
                                ));
                            }
                        }
                        _ => (),
                    },
                    ServerBoundPacket::MapSelect(new_map) => match &mut self.game_state.phase {
                        GamePhase::ConnectingAndChoosingSettings { player_choices, .. } => {
                            if let Some(PlayerChoices { map, .. }) = &mut player_choices[player_num]
                            {
                                println!(
                                    "Setting player #{}'s map vote to {}!",
                                    player_num,
                                    new_map.clone()
                                );
                                *map = new_map.clone();
                                need_to_broadcast
                                    .push(ClientBoundPacket::PlayerMapChoice(player_num, new_map));
                            }
                        }
                        _ => (),
                    },
                    ServerBoundPacket::SetReadyStatus(new_status) => {
                        match &mut self.game_state.phase {
                            GamePhase::ConnectingAndChoosingSettings { player_choices, .. } => {
                                if let Some(PlayerChoices { ready, .. }) =
                                    &mut player_choices[player_num]
                                {
                                    println!(
                                        "Player {} is no{} ready!",
                                        player_num,
                                        if new_status { "w" } else { "t" }
                                    );
                                    *ready = new_status;
                                    need_to_broadcast.push(ClientBoundPacket::PlayerReadyStatus(
                                        player_num, new_status,
                                    ));
                                }
                            }
                            _ => (),
                        }
                    }
                    ServerBoundPacket::ForceStart => {
                        if let GamePhase::ConnectingAndChoosingSettings { force_start, .. } =
                            &mut self.game_state.phase
                        {
                            *force_start = true;
                        }
                    }

                    ServerBoundPacket::NotifyLoaded => match &mut self.game_state.phase {
                        GamePhase::WaitingForPlayerLoad { players_loaded } => {
                            players_loaded[player_num] = true;
                        }
                        _ => (),
                    },

                    ServerBoundPacket::InputToggle(event) => match event {
                        InputEvent::Engine(status) => {
                            self.game_state.players[player_num]
                                .player_inputs
                                .engine_status = status;
                        }
                        InputEvent::Rotation(status) => {
                            self.game_state.players[player_num]
                                .player_inputs
                                .rotation_status = status;
                        }
                    },
                    ServerBoundPacket::NextGame => {
                        if let GamePhase::AllPlayersDone(placements) = self.game_state.phase {
                            println!("Starting next game!");
                            self.game_state.phase = GamePhase::ConnectingAndChoosingSettings {
                                force_start: false,
                                player_choices: placements
                                    .map(|opt| opt.map(|_| Default::default())), // TODO figure out previous settings?
                            };
                            self.game_state.map = None;
                            need_to_broadcast.push(ClientBoundPacket::StartNextGame);
                        }
                    }
                }
            }
        }
        for packet in need_to_broadcast {
            for conn in self.connections.iter_mut() {
                conn.push_outgoing(packet.clone());
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
        let now = Instant::now();
        match &mut self.game_state.phase {
            GamePhase::ConnectingAndChoosingSettings {
                force_start,
                player_choices,
            } => {
                if self.connections.len() < GLOBAL_CONFIG.player_amount
                    && !(*force_start && self.connections.len() > 0)
                {
                    match self.listener.accept() {
                        Ok((socket, addr)) => {
                            println!("new connection from {}", addr.ip().to_string());
                            let idx = self.connections.len();
                            self.connections.push(ClientConnection::new(socket));
                            self.connections.last_mut().unwrap().push_outgoing(
                                ClientBoundPacket::PlayerNumber(idx, player_choices.clone()),
                            );
                            player_choices[idx] = Some(Default::default());
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                        Err(e) => println!("couldn't get connecting client info {:?}", e),
                    }
                } else {
                    if player_choices.iter().enumerate().all(|(idx, r)| {
                        r.as_ref().map(|r| r.ready).unwrap_or(false)
                            || idx >= self.connections.len()
                    }) {
                        println!("Players ready! Loading...");
                        let map_name = GLOBAL_CONFIG.default_map_vote.clone(); // TODO figure out real voted map

                        for (player_num, conn) in self.connections.iter_mut().enumerate() {
                            self.game_state.players[player_num] =
                                get_player_start_physics_properties(
                                    &player_choices[player_num].as_ref().unwrap().chair,
                                    player_num,
                                );
                            conn.push_outgoing(ClientBoundPacket::LoadGame(map_name.clone()));
                        }

                        self.game_state.phase = GamePhase::WaitingForPlayerLoad {
                            players_loaded: player_choices
                                .iter()
                                .map(|x| if x.is_some() { false } else { true })
                                .collect::<Vec<bool>>()
                                .try_into()
                                .unwrap(),
                        };

                        self.game_state.map = Some(
                            Map::load(map_name).expect("Couldn't load the map on the server!"),
                        );
                    }
                }
            }

            GamePhase::WaitingForPlayerLoad { players_loaded } => {
                if players_loaded.iter().all(|&x| x) {
                    println!("Players loaded, getting ready...");
                    let time_until_start = Duration::new(3, 0);
                    self.game_state.phase =
                        GamePhase::CountingDownToGameStart(now + time_until_start);

                    for connection in &mut self.connections {
                        connection.push_outgoing(ClientBoundPacket::GameStart(time_until_start));
                    }
                }
            }

            GamePhase::CountingDownToGameStart(countdown_end_time) => {
                if now > *countdown_end_time {
                    println!("Go!!!");
                    // transition to playing game after countdown
                    self.game_state.phase = GamePhase::PlayingGame {
                        // start off with 10 seconds of vote free gameplay
                        voting_game_state: VotingState::VoteCooldown(now + Duration::new(10, 0)),
                        player_placement: [0, 1, 2, 3].map(|_| LapInformation::new()),
                        question_idx: 0,
                    }
                }
            }

            GamePhase::PlayingGame {
                voting_game_state,
                question_idx,
                ..
            } => {
                // update bounding box dimensions and temporary physics changes for each player
                for player in &mut self.game_state.players {
                    player
                        .physics_changes
                        .retain(|change| change.expiration_time > now);
                    player.update_bounding_box();
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

                self.game_state.players = [0, 1, 2, 3].map(|n| {
                    self.game_state.players[n].do_physics_step(
                        1.0,
                        others(n),
                        self.game_state
                            .map
                            .as_mut()
                            .expect("No map loaded in game loop!")
                            .trigger_iter(),
                    )
                });

                match &mut *voting_game_state {
                    VotingState::WaitingForVotes {
                        audience_votes,
                        vote_close_time,
                        current_question,
                    } => {
                        if *vote_close_time < now {
                            let mut counts = HashMap::new();
                            for vote in audience_votes {
                                *counts.entry(vote.1).or_insert(0) += 1;
                            }
                            let winner: usize = **counts
                                .iter()
                                .max_by(|a, b| a.1.cmp(&b.1))
                                .map(|(vote, _c)| vote)
                                .unwrap_or(&&mut (0));

                            GameServer::broadcast_ws(
                                &mut self.ws_connections,
                                WSAudienceBoundMessage::Winner(winner),
                            );

                            let decision = current_question.options[winner].clone();

                            for client in self.connections.iter_mut() {
                                client.push_outgoing(ClientBoundPacket::InteractionActivate(
                                    current_question.clone(),
                                    decision.clone(),
                                ));
                            }

                            let decision_end_time = now + Duration::new(30, 0);

                            match decision.action {
                                chariot_core::questions::AudienceAction::NoLeft => {
                                    self.game_state.players.iter_mut().for_each(|playa| {
                                        playa.physics_changes.push(PhysicsChange {
                                            change_type: PhysicsChangeType::NoTurningLeft,
                                            expiration_time: decision_end_time,
                                        });
                                    });
                                }
                                chariot_core::questions::AudienceAction::NoRight => {
                                    self.game_state.players.iter_mut().for_each(|playa| {
                                        playa.physics_changes.push(PhysicsChange {
                                            change_type: PhysicsChangeType::NoTurningRight,
                                            expiration_time: decision_end_time,
                                        });
                                    });
                                }
                            }

                            *voting_game_state = VotingState::VoteResultActive {
                                decision,
                                decision_end_time,
                            };
                        }
                    }
                    VotingState::VoteResultActive {
                        decision_end_time, ..
                    } => {
                        if *decision_end_time < now {
                            // the vote has been in effect enough, lets go to the cooldown
                            *voting_game_state =
                                VotingState::VoteCooldown(now + Duration::new(10, 0))
                        }
                    }
                    VotingState::VoteCooldown(cooldown) => {
                        if *cooldown < now {
                            let time_until_voting_enabled = Duration::new(30, 0);
                            let question: QuestionData = QUESTIONS[*question_idx].clone();
                            *question_idx = (*question_idx + 1) % QUESTIONS.len();

                            *voting_game_state = VotingState::WaitingForVotes {
                                audience_votes: HashMap::new(),
                                current_question: question.clone(),
                                vote_close_time: now + time_until_voting_enabled, // now + 30 seconds
                            };

                            for client in self.connections.iter_mut() {
                                client.push_outgoing(ClientBoundPacket::VotingStarted(
                                    question.clone(),
                                ));
                            }

                            GameServer::broadcast_ws(
                                &mut self.ws_connections,
                                WSAudienceBoundMessage::Prompt(question.clone()),
                            );
                        }
                    }
                }

                if self
                    .game_state
                    .players
                    .iter()
                    .enumerate()
                    .all(|(player_num, player)| {
                        player.lap_info.finished || player_num >= self.connections.len()
                    })
                {
                    let final_placements: [Placement; 4] = self
                        .game_state
                        .players
                        .iter()
                        .map(|player| player.lap_info.placement)
                        .collect::<Vec<Placement>>()
                        .try_into()
                        .unwrap();

                    for conn in &mut self.connections {
                        conn.push_outgoing(ClientBoundPacket::AllDone(final_placements.clone()));
                    }

                    self.game_state.phase =
                        GamePhase::AllPlayersDone([0, 1, 2, 3].map(|player_num| {
                            if player_num < self.connections.len() {
                                Some(final_placements[player_num])
                            } else {
                                None
                            }
                        }));
                }
            }

            GamePhase::AllPlayersDone(_placements) => {
                // Don't need anything?
            }
        }
    }

    // queue up sending updated game state
    fn sync_state(&mut self) {
        match self.game_state.phase {
            // These two phases have visible players
            GamePhase::CountingDownToGameStart(_) | GamePhase::PlayingGame { .. } => {
                self.sync_player_state()
            }
            _ => (),
        }

        self.update_and_sync_placement_state();
    }

    // send placement data to each client, if its changed
    fn update_and_sync_placement_state(&mut self) {
        if let Some(map) = &self.game_state.map {
            if let GamePhase::PlayingGame {
                player_placement, ..
            } = &mut self.game_state.phase
            {
                let new_placement_array: [(PlayerID, LapInformation); 4] =
                    get_player_placement_array(&self.game_state.players, &map.checkpoints);

                for &(player_num, lap_information @ LapInformation { lap, placement, .. }) in
                    new_placement_array.iter()
                {
                    if self.connections.len() <= player_num {
                        continue;
                    };

                    if player_placement[player_num].lap != lap {
                        self.connections[player_num]
                            .push_outgoing(ClientBoundPacket::LapUpdate(lap));
                    } else if player_placement[player_num].placement != placement {
                        // notify the player now in a different place that
                        // their new placement is different; the one that used
                        // to be there will get notified when it's their turn
                        self.connections[player_num]
                            .push_outgoing(ClientBoundPacket::PlacementUpdate(placement));
                    }

                    player_placement[player_num] = lap_information;
                }
            }
        }
    }

    // send player location and velocity data to every client
    fn sync_player_state(&mut self) {
        let updates: Vec<(EntityLocation, DVec3)> = self
            .game_state
            .players
            .iter()
            .map(|player| (player.entity_location, player.velocity))
            .collect();
        for connection in &mut self.connections {
            connection.push_outgoing(ClientBoundPacket::EntityUpdate(updates.clone()));
        }
    }
}
