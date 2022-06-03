use std::collections::HashMap;
use std::net::TcpListener;
use std::thread::{self};
use std::time::{Duration, Instant};

use chariot_core::player::choices::{Chair, PlayerChoices, Track};
use chariot_core::player::player_inputs::{EngineStatus, PlayerInputs};
use chariot_core::player::{lap_info::LapInformation, player_inputs::InputEvent, PlayerID};
use glam::DVec3;

use chariot_core::entity_location::EntityLocation;
use chariot_core::networking::ws::{QuestionResult, Standing, WSAudienceBoundMessage};
use chariot_core::networking::Uuid;
use chariot_core::networking::{
    ClientBoundPacket, ClientConnection, ServerBoundPacket, WebSocketConnection,
};
use chariot_core::player::lap_info::Placement;
use chariot_core::questions::{QuestionData, QUESTIONS};
use chariot_core::GLOBAL_CONFIG;

use crate::chairs::get_player_start_physics_properties;
use crate::physics::physics_changes::PhysicsChange;
use crate::physics::player_entity::PlayerEntity;
use crate::physics::ramp::RampCollisionResult;
use crate::progress::PlayerProgress;

use self::interactions::{
    get_physics_change_from_audience_action, get_stats_changes_from_audience_action,
    handle_one_time_audience_action,
};
use self::map::Map;
use self::phase::*;

mod interactions;
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
    tick_counter: u128,
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
        let ws_server =
            TcpListener::bind(&GLOBAL_CONFIG.ws_server_port).expect("could not bind to ws server");

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
                    .map(|num| get_player_start_physics_properties(&Chair::Swivel, num)),
                map: None,
            },
            tick_counter: 0,
        }
    }

    // WARNING: this function never returns
    pub fn start_loop(&mut self) {
        let max_server_tick_duration = Duration::from_millis(GLOBAL_CONFIG.server_tick_ms);

        loop {
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

            self.ws_connections.retain(|_, con| con.sync_outgoing());

            // wait until server tick time has elapsed
            if let Some(remaining_tick_duration) =
                max_server_tick_duration.checked_sub(start_time.elapsed())
            {
                thread::sleep(remaining_tick_duration);
            } else {
                match self.game_state.phase {
                    GamePhase::ConnectingAndChoosingSettings { .. }
                    | GamePhase::WaitingForPlayerLoad { .. } => println!("Tick took longer than configured length, but we don't care because we are still loading"),
                    _ => println!("server tick took longer than configured length"),
                }
            }

            self.tick_counter += 1;
        }
    }

    // handle every packet in received order
    fn process_incoming_packets(&mut self) {
        let mut need_to_broadcast: Vec<ClientBoundPacket> = vec![];
        let mut audience_need_to_broadcast: Vec<WSAudienceBoundMessage> = vec![];
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
                                audience_need_to_broadcast.push(WSAudienceBoundMessage::Standings(
                                    [0, 1, 2, 3].map(|idx| -> Standing {
                                        let mut chair = Chair::Swivel;
                                        if let Some(player_choice) = player_choices[idx].clone() {
                                            chair = player_choice.chair
                                        }
                                        Standing {
                                            name: idx.to_string(),
                                            chair: chair.to_string(),
                                            rank: (idx as u8) + 1,
                                            lap: 0,
                                        }
                                    }),
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
                            GamePhase::ConnectingAndChoosingSettings {
                                player_choices,
                                force_start,
                            } => {
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

                                    let players_in_game = player_choices.iter().flatten();
                                    let not_ready_players =
                                        players_in_game.clone().filter(|player| player.ready);

                                    if not_ready_players.count() == 0
                                        && players_in_game.count() == GLOBAL_CONFIG.player_amount
                                    {
                                        *force_start = true;
                                    }
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
                                player_choices: [
                                    Default::default(),
                                    Default::default(),
                                    Default::default(),
                                    Default::default(),
                                ],
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

        for packet in audience_need_to_broadcast {
            GameServer::broadcast_ws(&mut self.ws_connections, packet);
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
                            let idx = self.connections.len();
                            println!(
                                "new connection from {}, giving id {}",
                                addr.ip().to_string(),
                                idx
                            );
                            self.connections.push(ClientConnection::new(socket));
                            self.connections.last_mut().unwrap().push_outgoing(
                                ClientBoundPacket::PlayerNumber(idx, player_choices.clone()),
                            );
                            player_choices[idx] = Some(Default::default());

                            // go tell everyone else we have a new player
                            println!("telling everyone about our new player");
                            for con in self.connections.iter_mut() {
                                con.push_outgoing(ClientBoundPacket::PlayerJoined(idx));
                            }
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
                        let track = Track::Track; // TODO figure out real voted track

                        for (player_num, conn) in self.connections.iter_mut().enumerate() {
                            self.game_state.players[player_num] =
                                get_player_start_physics_properties(
                                    &player_choices[player_num].as_ref().unwrap().chair,
                                    player_num,
                                );
                            conn.push_outgoing(ClientBoundPacket::LoadGame(track.clone()));
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
                            Map::load(track.to_string())
                                .expect("Couldn't load the map on the server!"),
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
                    [0, 1, 2, 3].map(|i| {
                        self.game_state.players[i].placement_data = PlayerProgress::Racing {
                            lap_info: LapInformation::new(),
                        }
                    });
                    // transition to playing game after countdown
                    let vote_cooldown_time = now + Duration::new(10, 0);
                    self.game_state.phase = GamePhase::PlayingGame {
                        // start off with 10 seconds of vote free gameplay
                        voting_game_state: VotingState::VoteCooldown(vote_cooldown_time),
                        question_idx: 0,
                    };
                    // tell the audience that we have player placement now
                    GameServer::broadcast_ws(
                        &mut self.ws_connections,
                        WSAudienceBoundMessage::Standings([0, 1, 2, 3].map(|idx| -> Standing {
                            Standing {
                                name: idx.to_string(),
                                chair: self.game_state.players[idx].chair.to_string(),
                                rank: idx as u8,
                                lap: 1,
                            }
                        })),
                    );
                    GameServer::broadcast_ws(
                        &mut self.ws_connections,
                        WSAudienceBoundMessage::Countdown {
                            time: vote_cooldown_time,
                        },
                    )
                }
            }

            GamePhase::PlayingGame {
                voting_game_state,
                question_idx,
                ..
            } => {
                let player_placement_data = Self::find_player_placement(&self.game_state.players);

                for (player_id, placement) in player_placement_data {
                    if self.game_state.players[player_id].cached_place != Some(placement) {
                        self.game_state.players[player_id].cached_place = Some(placement);
                        if self.connections.len() > player_id {
                            self.connections[player_id]
                                .push_outgoing(ClientBoundPacket::PlacementUpdate(placement));
                        }
                    }

                    //println!("player #{player_id} in place {placement}");
                }

                let ramps = &self
                    .game_state
                    .map
                    .as_ref()
                    .expect("No map loaded in game loop!")
                    .ramps
                    .clone();

                let mut per_player_current_ramps: Vec<RampCollisionResult> = vec![];
                let mut original_player_inputs: Vec<PlayerInputs> = vec![];

                // update bounding box dimensions and temporary physics changes for each player
                for player in &mut self.game_state.players {
                    player.update_bounding_box();
                    player
                        .physics_changes
                        .retain(|change| change.expiration_time > now);
                    player
                        .stats_changes
                        .retain(|change| change.expiration_time > now);

                    original_player_inputs.push(player.player_inputs);
                    player.change_inputs_per_physics_changes();
                    let ramp_collision_result = player.update_upwards_from_ramps(ramps);
                    per_player_current_ramps.push(ramp_collision_result);
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

                let colliders = &self
                    .game_state
                    .map
                    .as_ref()
                    .expect("No map loaded in game loop!")
                    .colliders
                    .clone();

                let speedup_zones = &self
                    .game_state
                    .map
                    .as_ref()
                    .expect("No map loaded in game loop!")
                    .speedup_zones
                    .clone();

                self.game_state.players = [0, 1, 2, 3].map(|n| {
                    let mut player = self.game_state.players[n].do_physics_step(
                        1.0,
                        others(n),
                        colliders.clone(),
                        self.game_state
                            .map
                            .as_mut()
                            .expect("No map loaded in game loop!")
                            .trigger_iter(),
                        speedup_zones,
                        per_player_current_ramps.get(n).unwrap(),
                    );

                    if let PlayerProgress::Racing {
                        lap_info: LapInformation { lap: old_lap, .. },
                    } = self.game_state.players[n].placement_data
                    {
                        if let PlayerProgress::Racing {
                            lap_info: LapInformation { lap: new_lap, .. },
                        } = player.placement_data
                        {
                            if old_lap != new_lap {
                                self.connections[n]
                                    .push_outgoing(ClientBoundPacket::LapUpdate(new_lap));
                            }
                        }
                    }

                    // Restore original player inputs: without this, the server's inputs can change multiple times per client update
                    player.player_inputs = original_player_inputs.get(n).unwrap().to_owned();
                    player
                });

                match &mut *voting_game_state {
                    VotingState::WaitingForVotes {
                        audience_votes,
                        vote_close_time,
                        current_question,
                    } => {
                        if *vote_close_time < now {
                            let mut counts = HashMap::new();

                            let total_vote_count = audience_votes.len();

                            for vote in audience_votes {
                                *counts.entry(vote.1).or_insert(0) += 1;
                            }

                            let winner: usize = **counts
                                .iter()
                                .max_by(|a, b| a.1.cmp(&b.1))
                                .map(|(vote, _c)| vote)
                                .unwrap_or(&&mut (0));

                            let decision = current_question.options[winner].clone();
                            let time_effect_is_live = Duration::new(15, 0);
                            let effect_end_time = now + time_effect_is_live;

                            let option_results = current_question
                                .options
                                .iter()
                                .enumerate()
                                .map(|(idx, q)| {
                                    let percentage: f32 = if total_vote_count == 0 {
                                        if idx == winner {
                                            1.0 // default to 100% for the winning vote
                                        } else {
                                            0.0
                                        }
                                    } else {
                                        *counts.get(&idx).unwrap_or(&0) as f32
                                            / total_vote_count as f32
                                    };

                                    QuestionResult {
                                        label: q.label.clone(),
                                        percentage,
                                    }
                                })
                                .collect();

                            GameServer::broadcast_ws(
                                &mut self.ws_connections,
                                WSAudienceBoundMessage::Winner {
                                    choice: winner,
                                    vote_effect_time: effect_end_time,
                                    option_results,
                                },
                            );

                            for client in self.connections.iter_mut() {
                                client.push_outgoing(ClientBoundPacket::InteractionActivate {
                                    question: current_question.clone(),
                                    decision: decision.clone(),
                                    time_effect_is_live,
                                    winner_idx: winner,
                                });
                            }

                            if let Some(change_type) =
                                get_physics_change_from_audience_action(&decision.action)
                            {
                                let change = PhysicsChange {
                                    change_type,
                                    expiration_time: effect_end_time,
                                };
                                self.game_state.players.iter_mut().for_each(|player| {
                                    player.physics_changes.push(change.clone());
                                });
                            }

                            for change in get_stats_changes_from_audience_action(
                                &decision.action,
                                effect_end_time,
                            ) {
                                self.game_state.players.iter_mut().for_each(|player| {
                                    player.stats_changes.push(change.clone());
                                })
                            }

                            handle_one_time_audience_action(
                                &decision.action,
                                &mut self.game_state.players,
                            );

                            for (n, conn) in self.connections.iter_mut().enumerate() {
                                if let PlayerProgress::Racing {
                                    lap_info: LapInformation { lap, .. },
                                } = self.game_state.players[n].placement_data
                                {
                                    conn.push_outgoing(ClientBoundPacket::LapUpdate(lap));
                                }
                            }

                            *voting_game_state = VotingState::VoteResultActive {
                                decision,
                                decision_end_time: effect_end_time,
                            };
                        } else if self.tick_counter % 10 == 0 {
                            let counts = self.get_vote_counts();
                            for conn in self.connections.iter_mut() {
                                conn.push_outgoing(ClientBoundPacket::VotingUpdate(counts.clone()));
                            }
                        }
                    }
                    VotingState::VoteResultActive {
                        decision_end_time, ..
                    } => {
                        if *decision_end_time < now {
                            // the vote has been in effect enough, lets go to the cooldown
                            let time_voting_starts = now + Duration::new(5, 0);
                            *voting_game_state = VotingState::VoteCooldown(time_voting_starts);
                            for client in self.connections.iter_mut() {
                                client.push_outgoing(ClientBoundPacket::VotingCooldown);
                            }
                            GameServer::broadcast_ws(
                                &mut self.ws_connections,
                                WSAudienceBoundMessage::Countdown {
                                    time: time_voting_starts,
                                },
                            )
                        }
                    }
                    VotingState::VoteCooldown(cooldown) => {
                        if *cooldown < now {
                            let time_until_vote_end = Duration::new(15, 0);
                            let vote_end_time = now + time_until_vote_end;
                            let question: QuestionData = QUESTIONS[*question_idx].clone();
                            *question_idx = (*question_idx + 1) % QUESTIONS.len();

                            *voting_game_state = VotingState::WaitingForVotes {
                                audience_votes: HashMap::new(),
                                current_question: question.clone(),
                                vote_close_time: vote_end_time,
                            };

                            for client in self.connections.iter_mut() {
                                client.push_outgoing(ClientBoundPacket::VotingStarted {
                                    question: question.clone(),
                                    time_until_vote_end,
                                });
                            }

                            GameServer::broadcast_ws(
                                &mut self.ws_connections,
                                WSAudienceBoundMessage::Prompt {
                                    vote_close_time: vote_end_time,
                                    question: question.clone(),
                                },
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
                        if let PlayerProgress::Finished { .. } = player.placement_data {
                            true
                        } else {
                            player_num >= self.connections.len()
                        }
                    })
                {
                    // Vec<(PlayerID, finish_time)>
                    let mut final_times = self
                        .game_state
                        .players
                        .iter()
                        .map(|player| {
                            if let PlayerProgress::Finished { finish_time } = player.placement_data
                            {
                                finish_time
                            } else {
                                // this is the site where josh lost all faith in git
                                Duration::from_secs(1000)
                            }
                        })
                        .enumerate()
                        .collect::<Vec<(usize, Duration)>>();
                    final_times.sort_by(|&(id, dur), (id2, dur2)| dur.cmp(dur2));

                    // Vec<(Placement, (PlayerID, finish_time))>
                    let mut final_places = final_times
                        .iter()
                        .enumerate()
                        .map(|(zero_based_place, &data)| ((zero_based_place + 1) as u8, data))
                        .collect::<Vec<(u8, (usize, Duration))>>();
                    // (
                    final_places.sort_by(|x, y| x.1 .0.cmp(&y.1 .0));

                    // Vec<(Placement, finish_time)>
                    let final_placement: [(u8, (u64, u32)); 4] = final_places
                        .iter()
                        .map(|(place, (player_id, finish_time))| {
                            (*place, (finish_time.as_secs(), finish_time.subsec_nanos()))
                        })
                        .collect::<Vec<(u8, (u64, u32))>>()
                        .try_into()
                        .unwrap();

                    for (playa, (placement, time)) in final_placement.iter().enumerate() {
                        println!(
                            "player {playa} finished {placement} in {}:{}!",
                            time.0, time.1
                        );
                    }
                    for conn in &mut self.connections {
                        conn.push_outgoing(ClientBoundPacket::AllDone {
                            placements: final_placement.clone(),
                        });
                    }

                    self.game_state.phase = GamePhase::AllPlayersDone(final_placement);
                }
            }

            GamePhase::AllPlayersDone(_placements) => {
                // Don't need anything?
            }
        }
    }

    fn find_player_placement(players: &[PlayerEntity; 4]) -> [(PlayerID, Placement); 4] {
        let mut placement = [0, 1, 2, 3]
            .map(|player_id| players[player_id].placement_data)
            .iter()
            .enumerate()
            .map(|(id, &d)| (id, d))
            .collect::<Vec<(PlayerID, PlayerProgress)>>();
        placement.sort_by(|x, y| x.1.cmp(&y.1));
        placement
            .iter()
            .enumerate()
            .map(|(place, (playerid, _))| (*playerid as PlayerID, (place + 1) as Placement))
            .collect::<Vec<(PlayerID, Placement)>>()
            .try_into()
            .unwrap()
    }

    // queue up sending updated game state
    fn sync_state(&mut self) {
        match self.game_state.phase {
            // These two phases have visible players
            GamePhase::CountingDownToGameStart(_) => self.sync_player_state(),
            GamePhase::PlayingGame { .. } => {
                self.sync_player_state();
                self.update_and_sync_placement_state();
            }
            _ => (),
        }

        self.sync_sfx_state();
    }

    // send placement data to each client, if its changed
    fn update_and_sync_placement_state(&mut self) {}

    // send player location and velocity data to every client
    fn sync_player_state(&mut self) {
        let updates: Vec<(EntityLocation, DVec3, bool)> = self
            .game_state
            .players
            .iter()
            .map(|player| {
                (
                    player.entity_location,
                    player.velocity,
                    matches!(
                        player.player_inputs.engine_status,
                        EngineStatus::Accelerating(_)
                    ),
                )
            })
            .collect();
        for connection in &mut self.connections {
            connection.push_outgoing(ClientBoundPacket::EntityUpdate(updates.clone()));
        }
    }

    fn sync_sfx_state(&mut self) {
        for (idx, connection) in &mut self.connections.iter_mut().enumerate() {
            for &effect in &self.game_state.players.get(idx).unwrap().sound_effects {
                connection.push_outgoing(ClientBoundPacket::SoundEffectEvent(effect));
            }
        }
    }
}
