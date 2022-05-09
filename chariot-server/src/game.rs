use std::collections::HashMap;
use std::net::TcpListener;
use std::ops::Add;
use std::thread::{self};
use std::time::{Duration, Instant};

use chariot_core::networking::ws::{QuestionBody, WSAudienceBoundMessage, WSServerBoundMessage};
use chariot_core::networking::Uuid;
use chariot_core::networking::{
    ClientBoundPacket, ClientConnection, ServerBoundPacket, WebSocketConnection,
};
use chariot_core::player_inputs::InputEvent;
use chariot_core::GLOBAL_CONFIG;

use crate::chairs::get_player_start_physics_properties;
use crate::physics::player_entity::PlayerEntity;

pub struct GameServer {
    listener: TcpListener,
    ws_server: TcpListener,
    connections: Vec<ClientConnection>,
    ws_connections: HashMap<Uuid, WebSocketConnection>,
    game_state: ServerGameState,
}

#[derive(PartialEq)]
enum VotingGameState {
    Voting,
    Waiting,
}

pub struct ServerGameState {
    players_ready: [bool; 4],
    new_players_joined: Vec<usize>,
    players: [PlayerEntity; 4],
    audience_votes: HashMap<Uuid, i32>,
    current_question: QuestionBody,
    voting_game_state: VotingGameState,
    vote_close_time: Instant,
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
                players_ready: [false, false, false, false],
                new_players_joined: Vec::new(),
                players: [0, 1, 2, 3]
                    .map(|num| get_player_start_physics_properties(&String::from("standard"), num)),
                audience_votes: HashMap::new(),
                voting_game_state: VotingGameState::Waiting,
                vote_close_time: Instant::now(),
                current_question: (
                    "q".to_string(),
                    (
                        "1".to_string(),
                        "2".to_string(),
                        "3".to_string(),
                        "4".to_string(),
                    ),
                ),
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

    // creates a websocket for any audience connections
    fn acquire_any_audience_connections(&mut self) {
        self.ws_server
            .set_nonblocking(true)
            .expect("non blocking should be ok");

        let mut new_uuids: Vec<Uuid> = Vec::new();

        if let Some(stream_result) = self.ws_server.incoming().next() {
            if let Ok(stream) = stream_result {
                let id = Uuid::new_v4();
                let connection = WebSocketConnection::new(stream);
                self.ws_connections.insert(id, connection);
                new_uuids.push(id);
                println!("acquired an audience connection!");
            }
        }

        self.ws_server
            .set_nonblocking(false)
            .expect("non blocking should be ok");

        for id in new_uuids {
            let conn = self.ws_connections.get_mut(&id).unwrap();

            conn.push_outgoing_messge(WSAudienceBoundMessage::Assignment(id));
            if self.game_state.voting_game_state == VotingGameState::Voting {
                conn.push_outgoing_messge(WSAudienceBoundMessage::Prompt(
                    self.game_state.current_question.clone(),
                ));
            }
        }
    }

    // handle every packet in received order
    fn process_incoming_packets(&mut self) {
        for (i, connection) in self.connections.iter_mut().enumerate() {
            while let Some(packet) = connection.pop_incoming() {
                match packet {
                    ServerBoundPacket::ChairSelectAndReady(chair_name) => {
                        self.game_state.new_players_joined.push(i);
                        self.game_state.players[i] =
                            get_player_start_physics_properties(&chair_name, i.try_into().unwrap());
                    }
                    ServerBoundPacket::InputToggle(event) => match event {
                        InputEvent::Engine(status) => {
                            assert!(self.game_state.players_ready[i]);
                            self.game_state.players[i].player_inputs.engine_status = status;
                            println!("Engine status: {:?}", status);
                        }
                        InputEvent::Rotation(status) => {
                            assert!(self.game_state.players_ready[i]);
                            self.game_state.players[i].player_inputs.rotation_status = status;
                            println!("Turn status: {:?}", status);
                        }
                    },
                }
            }
        }
    }

    // handle socket data
    fn process_ws_packets(&mut self) {
        for (_id, connection) in self.ws_connections.iter_mut() {
            while let Some(packet) = connection.pop_incoming() {
                match packet {
                    WSServerBoundMessage::Vote(id, vote) => {
                        println!("{} voted for {}", id, vote);
                        self.game_state.audience_votes.insert(id, vote);
                    }
                }
            }
        }
    }

    // check to see if we need to tally up votes and do something
    fn check_audience_voting(&mut self) {
        if self.game_state.voting_game_state == VotingGameState::Voting
            && self.game_state.vote_close_time < Instant::now()
        {
            // time to tally up votes
            let winner = self
                .game_state
                .audience_votes
                .iter()
                .max_by(|a, b| a.1.cmp(&b.1))
                .map(|(_key, vote)| vote)
                .unwrap_or(&0);

            println!("Option {} won!", winner);
            self.game_state.voting_game_state = VotingGameState::Waiting;
            GameServer::broadcast_ws(
                &mut self.ws_connections,
                WSAudienceBoundMessage::Winner(*winner),
            );
        }
    }

    // prompts for a question
    fn start_audience_voting(
        &mut self,
        question: String,
        option1: String,
        option2: String,
        option3: String,
        option4: String,
        poll_time: Duration,
    ) {
        self.game_state.current_question = (question, (option1, option2, option3, option4));

        GameServer::broadcast_ws(
            &mut self.ws_connections,
            WSAudienceBoundMessage::Prompt(self.game_state.current_question.clone()),
        );

        self.game_state.audience_votes = HashMap::new(); // clear past votes
        self.game_state.voting_game_state = VotingGameState::Voting;
        self.game_state.vote_close_time = Instant::now().add(poll_time);
        // check on votes in 30 seconds
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

        // Add any new players
        while let Some(index) = self.game_state.new_players_joined.pop() {
            self.game_state.players_ready[index] = true;
            self.connections[index].push_outgoing(ClientBoundPacket::PlayerNumber(index as u8));
        }

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
                .filter(|(other_index, _)| self.game_state.players_ready[*other_index])
                .map(|(_, player_entity)| player_entity)
                .collect()
        };

        self.game_state.players =
            [0, 1, 2, 3].map(|n| self.game_state.players[n].do_physics_step(1.0, others(n)));
    }

    // queue up sending updated game state
    fn sync_state(&mut self) {
        for connection in &mut self.connections {
            let locations = [0, 1, 2, 3].map(|n| {
                if self.game_state.players_ready[n] {
                    Some(self.game_state.players[n].entity_location.clone())
                } else {
                    None
                }
            });

            connection.push_outgoing(ClientBoundPacket::LocationUpdate(locations));
        }
    }
}
