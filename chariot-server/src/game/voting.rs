use std::collections::HashMap;
use std::ops::Add;
use std::time::{Duration, Instant};

use chariot_core::networking::ws::{WSAudienceBoundMessage, WSServerBoundMessage};
use chariot_core::networking::Uuid;
use chariot_core::networking::WebSocketConnection;

use crate::game::phase::VotingState;
use crate::game::GameServer;

use super::phase::GamePhase;

impl GameServer {
    // handle socket data
    pub fn process_ws_packets(&mut self) {
        for (_id, connection) in self.ws_connections.iter_mut() {
            while let Some(packet) = connection.pop_incoming() {
                match packet {
                    WSServerBoundMessage::Vote(id, vote) => {
                        if let GamePhase::PlayingGame(game_state) = &mut self.game_state.phase {
                            if let VotingState::WaitingForVotes(state) =
                                &mut game_state.voting_game_state
                            {
                                println!("{} voted for {}", id, vote);
                                state.audience_votes.insert(id, vote);
                            }
                        }
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

            conn.push_outgoing_message(WSAudienceBoundMessage::Assignment(id));

            if let GamePhase::PlayingGame(game_state) = &mut self.game_state.phase {
                if let VotingState::WaitingForVotes(state) = &mut game_state.voting_game_state {
                    conn.push_outgoing_message(WSAudienceBoundMessage::Prompt(
                        state.current_question.clone(),
                    ))
                }
            }
        }
    }
}
