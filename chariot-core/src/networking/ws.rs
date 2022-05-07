use std::collections::VecDeque;
use std::net::TcpStream;
pub use tungstenite::{accept, Message, WebSocket};

pub struct WSConnection {
    socket: WebSocket<TcpStream>,
    incoming_packets: VecDeque<Message>,
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
                    self.incoming_packets.push_back(msg);
                }
            }
            Err(_) => {}
        }
    }

    pub fn pop_incoming(&mut self) -> Option<Message> {
        self.incoming_packets.pop_front()
    }

    pub fn push_outgoing(&mut self, packet: Message) -> () {
        self.outgoing_packets.push_back(packet);
    }

    // send packets on this connection until exhausted
    pub fn sync_outgoing(&mut self) {
        println!("{}", self.outgoing_packets.len());
        while let Some(msg) = self.outgoing_packets.pop_front() {
            if (self.socket.can_write()) {
                self.socket
                    .write_message(msg)
                    .expect("should have been able to send message");
            }
        }
    }
}
