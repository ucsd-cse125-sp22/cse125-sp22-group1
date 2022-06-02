use serde::{Deserialize, Serialize};
use serde_json::Error;
use std::net::TcpStream;
use std::{collections::VecDeque, time::Instant};
pub use tungstenite::{accept, Message, WebSocket};
pub use uuid::Uuid;

use crate::questions::QuestionData;

#[derive(Serialize, Deserialize, Clone)]
pub struct Standing {
    pub name: String,
    pub chair: String,
    pub rank: u8,
    pub lap: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct QuestionResult {
    pub label: String,
    pub percentage: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum WSAudienceBoundMessage {
    Prompt {
        question: QuestionData,
        #[serde(with = "serde_millis")]
        vote_close_time: Instant,
    }, // QuestionData, Time Until Vote Close

    Winner {
        choice: usize,
        option_results: Vec<QuestionResult>, // percentages of all the winners
        #[serde(with = "serde_millis")]
        vote_effect_time: Instant,
    }, // The winning choice (tuple index)

    Assignment(Uuid), // Sends a uuid that the server will use to identify the client

    Standings([Standing; 4]),
    Countdown {
        #[serde(with = "serde_millis")]
        time: Instant,
    },
    AudienceCount(usize), // The number of connections to the audience
}

#[derive(Serialize, Deserialize)]
pub enum WSServerBoundMessage {
    Vote(Uuid, usize), // Client UUID, the option to vote for
}

pub struct WSConnection {
    socket: WebSocket<TcpStream>,
    incoming_packets: VecDeque<WSServerBoundMessage>,
    outgoing_packets: VecDeque<Message>,
}

impl WSConnection {
    pub fn new(tcp_stream: TcpStream) -> Option<WSConnection> {
        tcp_stream
            .set_nonblocking(false)
            .expect("expected to be able to set tcp nonblocking to false");
        match accept(tcp_stream) {
            Ok(socket) => {
                socket
                    .get_ref()
                    .set_nonblocking(true)
                    .expect("expected to be able to set tcp nonblocking to true");
                return Some(WSConnection {
                    socket,
                    incoming_packets: VecDeque::new(),
                    outgoing_packets: VecDeque::new(),
                });
            }
            Err(err) => {
                println!("something weird happened re web sockets; we don't really care though — error: {err}");
                None
            }
        }
    }

    pub fn fetch_incoming_packets(&mut self) {
        if let Ok(msg) = self.socket.read_message() {
            if msg.is_text() {
                if let Ok(txt) = msg.to_text() {
                    // this is where we handle shit
                    let message_result: Result<WSServerBoundMessage, Error> =
                        serde_json::from_str(txt);

                    match message_result {
                        Ok(server_bound_message) => {
                            self.incoming_packets.push_back(server_bound_message)
                        }
                        Err(err) => {
                            println!("got an error! we're going to do nothing about this!");
                            println!("{}", err);
                        }
                    }
                }
            }
        }
    }

    pub fn pop_incoming(&mut self) -> Option<WSServerBoundMessage> {
        self.incoming_packets.pop_front()
    }

    pub fn push_outgoing(&mut self, packet: Message) -> () {
        self.outgoing_packets.push_back(packet);
    }

    pub fn push_outgoing_message(&mut self, packet: WSAudienceBoundMessage) -> () {
        let json_string =
            serde_json::to_string(&packet).expect("should have been able to serialize packet");
        let message = Message::Text(json_string);
        self.push_outgoing(message);
    }

    // send packets on this connection until exhausted
    pub fn sync_outgoing(&mut self) -> bool {
        let mut could_send_messages = true;
        while let Some(msg) = self.outgoing_packets.pop_front() {
            if self.socket.can_write() {
                let result = self.socket.write_message(msg);
                if result.is_err() {
                    println!(
                        "failed to write to socket because of {}",
                        result.unwrap_err()
                    );
                    could_send_messages = false;
                }
            } else {
                println!("couldn't write? :thinking_emoji");
                could_send_messages = false;
            }
        }
        could_send_messages
    }
}
