use std::collections::HashMap;

use chariot_core::networking::ws::{
    QuestionResult, Standing, WSAudienceBoundMessage, WSServerBoundMessage,
};
use chariot_core::networking::Uuid;
use chariot_core::networking::WebSocketConnection;
use chariot_core::questions::QuestionOption;

use crate::game::phase::VotingState;
use crate::game::GameServer;

use super::phase::GamePhase;

pub type QuestionID = usize;
pub type AnswerID = usize;

impl GameServer {
    // handle socket data
    pub fn process_ws_packets(&mut self) {
        for (_id, connection) in self.ws_connections.iter_mut() {
            while let Some(packet) = connection.pop_incoming() {
                match packet {
                    WSServerBoundMessage::Vote(id, vote) => {
                        if let GamePhase::PlayingGame {
                            voting_game_state, ..
                        } = &mut self.game_state.phase
                        {
                            if let VotingState::WaitingForVotes { audience_votes, .. } =
                                voting_game_state
                            {
                                println!("{} voted for {}", id, vote);
                                audience_votes.insert(id, vote);
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
                if let Some(connection) = connection {
                    self.ws_connections.insert(id, connection);
                    new_uuids.push(id);
                    println!("acquired an audience connection!");
                }
            }
        }

        self.ws_server
            .set_nonblocking(false)
            .expect("non blocking should be ok");

        for id in new_uuids {
            let conn = self.ws_connections.get_mut(&id).unwrap();

            conn.push_outgoing_message(WSAudienceBoundMessage::Assignment(id));

            if let GamePhase::PlayingGame {
                voting_game_state,
                player_placement,
                ..
            } = &mut self.game_state.phase
            {
                conn.push_outgoing_message(WSAudienceBoundMessage::Standings([0, 1, 2, 3].map(
                    |idx| -> Standing {
                        Standing {
                            name: idx.to_string(),
                            chair: self.game_state.players[idx].chair.to_string(),
                            rank: player_placement[idx].placement,
                            lap: player_placement[idx].lap,
                        }
                    },
                )));

                if let VotingState::WaitingForVotes {
                    current_question,
                    vote_close_time,
                    ..
                } = voting_game_state
                {
                    conn.push_outgoing_message(WSAudienceBoundMessage::Prompt {
                        question: current_question.clone(),
                        vote_close_time: vote_close_time.clone(),
                    })
                } else if let VotingState::VoteResultActive {
                    decision_end_time, ..
                } = voting_game_state
                {
                    conn.push_outgoing_message(WSAudienceBoundMessage::Countdown {
                        time: decision_end_time.clone(),
                    });
                }
            } else {
                conn.push_outgoing_message(WSAudienceBoundMessage::Standings([0, 1, 2, 3].map(
                    |idx| -> Standing {
                        Standing {
                            name: idx.to_string(),
                            chair: self.game_state.players[idx].chair.to_string(),
                            rank: (idx as u8) + 1,
                            lap: 0,
                        }
                    },
                )));
            }
        }

        let total_connections = self.ws_connections.len();
        GameServer::broadcast_ws(
            &mut self.ws_connections,
            WSAudienceBoundMessage::AudienceCount(total_connections),
        )
    }

    // depending on the game state, this function will maybe get the voting state
    pub fn get_vote_counts(&self) -> Vec<u32> {
        if let GamePhase::PlayingGame {
            voting_game_state, ..
        } = &self.game_state.phase
        {
            if let VotingState::WaitingForVotes {
                audience_votes,
                current_question,
                ..
            } = voting_game_state
            {
                let mut counts = HashMap::new();

                for vote in audience_votes {
                    *counts.entry(vote.1).or_insert(0) += 1;
                }

                let winner: usize = **counts
                    .iter()
                    .max_by(|a, b| a.1.cmp(&b.1))
                    .map(|(vote, _c)| vote)
                    .unwrap_or(&&(0 as usize));

                let option_results: Vec<u32> = current_question
                    .options
                    .iter()
                    .enumerate()
                    .map(|(idx, _q)| {
                        if idx == winner {
                            1
                        } else {
                            *counts.get(&idx).unwrap_or(&0) as u32
                        }
                    })
                    .collect();

                println!("game results!");
                println!("{:?}", option_results);
                return option_results;
            }
        }

        let counts: Vec<u32> = vec![
            rand::random::<u32>() % 50,
            rand::random::<u32>() % 50,
            rand::random::<u32>() % 50,
            rand::random::<u32>() % 50,
        ];

        return counts;
    }
}
