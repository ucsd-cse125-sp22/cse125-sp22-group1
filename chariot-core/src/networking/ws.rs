use serde_json::Error;
use std::collections::VecDeque;
use std::net::TcpStream;
pub use tungstenite::{accept, Message, WebSocket};

use super::{WSAudienceBoundMessage, WSServerBoundMessage};

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
        let msg_result = self.socket.read_message();
        match msg_result {
            Ok(msg) => {
                if msg.is_binary() || msg.is_text() {
                    // this is where we handle shit
                    let txt = msg
                        .to_text()
                        .expect("should have been able to convert message to string");
                    let message_result: Result<WSServerBoundMessage, Error> =
                        serde_json::from_str(txt);
                    if message_result.is_err() {
                        // self.incoming_packets.push_back(msg);
                    } else {
                        self.incoming_packets.push_back(message_result.unwrap());
                    }
                }
            }
            Err(_) => {}
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
