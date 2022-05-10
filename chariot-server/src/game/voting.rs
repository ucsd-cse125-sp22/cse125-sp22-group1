use std::collections::HashMap;
use std::ops::Add;
use std::time::{Duration, Instant};

use chariot_core::networking::ws::{WSAudienceBoundMessage, WSServerBoundMessage};
use chariot_core::networking::Uuid;
use chariot_core::networking::WebSocketConnection;

use crate::game::GameServer;

use super::phase::GamePhase;

impl GameServer {
    // handle socket data
    pub fn process_ws_packets(&mut self) {
        for (_id, connection) in self.ws_connections.iter_mut() {
            while let Some(packet) = connection.pop_incoming() {
                match packet {
                    WSServerBoundMessage::Vote(id, vote) => {
                        println!("{} voted for {}", id, vote);
                        self.game_state
                            .playing_with_voting_state
                            .audience_votes
                            .insert(id, vote);
                    }
                }
            }
        }
    }

    // creates a websocket for any audience connections
    pub fn acquire_any_audience_connections(&mut self) {
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

            if matches!(self.game_state.phase, GamePhase::PlayingWithVoting) {
                if self.game_state.playing_with_voting_state.is_voting_ongoing {
                    conn.push_outgoing_messge(WSAudienceBoundMessage::Prompt(
                        self.game_state
                            .playing_with_voting_state
                            .current_question
                            .clone(),
                    ));
                }
            }
        }
    }

    // check to see if we need to tally up votes and do something
    pub fn check_audience_voting(&mut self) {
        let state = &mut self.game_state.playing_with_voting_state;
        if state.is_voting_ongoing && state.vote_close_time < Instant::now() {
            // time to tally up votes
            let winner = state
                .audience_votes
                .iter()
                .max_by(|a, b| a.1.cmp(&b.1))
                .map(|(_key, vote)| vote)
                .unwrap_or(&0);

            println!("Option {} won!", winner);
            state.is_voting_ongoing = false;
            GameServer::broadcast_ws(
                &mut self.ws_connections,
                WSAudienceBoundMessage::Winner(*winner),
            );
        }
    }

    // prompts for a question
    pub fn start_audience_voting(
        &mut self,
        question: String,
        option1: String,
        option2: String,
        option3: String,
        option4: String,
        poll_time: Duration,
    ) {
        let state = &mut self.game_state.playing_with_voting_state;
        state.current_question = (question, (option1, option2, option3, option4));

        GameServer::broadcast_ws(
            &mut self.ws_connections,
            WSAudienceBoundMessage::Prompt(state.current_question.clone()),
        );

        state.audience_votes = HashMap::new(); // clear past votes
        state.is_voting_ongoing = true;
        state.vote_close_time = Instant::now().add(poll_time);
        // check on votes in 30 seconds
    }
}
