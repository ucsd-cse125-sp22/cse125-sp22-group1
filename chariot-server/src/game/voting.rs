use chariot_core::networking::ws::{WSAudienceBoundMessage, WSServerBoundMessage};
use chariot_core::networking::Uuid;
use chariot_core::networking::WebSocketConnection;

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
        let mut new_uuids: Vec<Uuid> = Vec::new();

        for stream in self.ws_server.incoming() {
            match stream {
                Ok(stream) => {
                    let acceptor = self.acceptor.clone();

                    match acceptor.accept(stream) {
                        Ok(stream) => {
                            let id = Uuid::new_v4();
                            let connection = WebSocketConnection::new(stream);
                            self.ws_connections.insert(id, connection);
                            new_uuids.push(id);
                            println!("acquired an audience connection!");
                        }
                        Err(error) => {
                            if let openssl::ssl::HandshakeError::WouldBlock(stream) = error {
                                println!("{:?}", stream.error());
                            } else {
                                println!("fuck — {:?}", error);
                            }
                        }
                    }
                    println!("big we have a stream");
                }
                Err(e) => {
                    println!("this shit too hard {:?}", e);
                }
            }
        }

        self.ws_server
            .set_nonblocking(true)
            .expect("non blocking should be ok");

        // if let Some(stream_result) = self.ws_server.incoming().next() {
        //     if let Ok(stream) = stream_result {
        //         println!("we got a stream! {:?}", stream);
        //         let acceptor = self.acceptor.clone();
        //         if let Ok(stream) = acceptor.accept(stream) {
        //             let id = Uuid::new_v4();
        //             let connection = WebSocketConnection::new(stream);
        //             self.ws_connections.insert(id, connection);
        //             new_uuids.push(id);
        //             println!("acquired an audience connection!");
        //         }
        //     }
        // }

        self.ws_server
            .set_nonblocking(false)
            .expect("non blocking should be ok");

        for id in new_uuids {
            let conn = self.ws_connections.get_mut(&id).unwrap();

            conn.push_outgoing_message(WSAudienceBoundMessage::Assignment(id));

            if let GamePhase::PlayingGame {
                voting_game_state, ..
            } = &mut self.game_state.phase
            {
                if let VotingState::WaitingForVotes {
                    current_question, ..
                } = voting_game_state
                {
                    conn.push_outgoing_message(WSAudienceBoundMessage::Prompt(
                        current_question.clone(),
                    ))
                }
            }
        }
    }
}
