use serde::{Deserialize, Serialize};
use serde_json::Error;
use std::collections::VecDeque;
use std::net::TcpStream;
pub use tungstenite::{accept, Message, WebSocket};
pub use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub enum WSAudienceBoundMessage {
    Prompt(QuestionBody), // Question, 4 Answer Choices

    Winner(i32), // The winning choice (tuple index)

    Assignment(Uuid), // Sends a uuid that the server will use to identify the client
}

pub type QuestionBody = (String, (String, String, String, String));

#[derive(Serialize, Deserialize)]
pub enum WSServerBoundMessage {
    Vote(Uuid, i32), // Client UUID, the option to vote for
}

pub struct WSConnection {
    socket: WebSocket<TcpStream>,
    incoming_packets: VecDeque<WSServerBoundMessage>,
    outgoing_packets: VecDeque<Message>,
}

impl WSConnection {
    pub fn new(tcp_stream: TcpStream) -> WSConnection {
        tcp_stream
            .set_nonblocking(false)
            .expect("expected to be able to set tcp nonblocking to false");
        match accept(tcp_stream) {
            Ok(socket) => {
                socket
                    .get_ref()
                    .set_nonblocking(true)
                    .expect("expected to be able to set tcp nonblocking to true");
                return WSConnection {
                    socket,
                    incoming_packets: VecDeque::new(),
                    outgoing_packets: VecDeque::new(),
                };
            }
            Err(err) => {
                panic!("problem — {:?}", err);
            }
        }
    }

    pub fn fetch_incoming_packets(&mut self) {
        if let Ok(msg) = self.socket.read_message() {
            if msg.is_text() && let Ok(txt) = msg.to_text() {
                // this is where we handle shit
                let message_result: Result<WSServerBoundMessage, Error> = serde_json::from_str(txt);

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

    pub fn pop_incoming(&mut self) -> Option<WSServerBoundMessage> {
        self.incoming_packets.pop_front()
    }

    pub fn push_outgoing(&mut self, packet: Message) -> () {
        self.outgoing_packets.push_back(packet);
    }

    pub fn push_outgoing_messge(&mut self, packet: WSAudienceBoundMessage) -> () {
        let json_string =
            serde_json::to_string(&packet).expect("should have been able to serialize packet");
        let message = Message::Text(json_string);
        self.push_outgoing(message);
    }

    // send packets on this connection until exhausted
    pub fn sync_outgoing(&mut self) {
        while let Some(msg) = self.outgoing_packets.pop_front() {
            if self.socket.can_write() {
                self.socket
                    .write_message(msg)
                    .expect("should have been able to send message");
            }
        }
    }
}
