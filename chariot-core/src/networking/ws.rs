use tungstenite::{accept, Message, WebSocket};

use std::collections::VecDeque;
use std::net::TcpStream;

pub struct WSConnection {
    socket: WebSocket<TcpStream>,
    incoming_packets: VecDeque<Message>,
    outgoing_packets: VecDeque<Message>,
}

impl WSConnection {
    pub fn new(tcp_stream: TcpStream) -> WSConnection {
        tcp_stream
            .set_nonblocking(false)
            .expect("should have been able to set nonblocking to false");
        match accept(tcp_stream) {
            Ok(socket) => {
                socket
                    .get_ref()
                    .set_nonblocking(true)
                    .expect("should have been able to set nonblocking to true");
                return WSConnection {
                    socket,
                    incoming_packets: VecDeque::new(),
                    outgoing_packets: VecDeque::new(),
                };
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }

    pub fn sync_incoming(&mut self) {
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
        while let Some(msg) = self.outgoing_packets.pop_front() {
            self.socket
                .write_message(msg)
                .expect("should have been able to send message");
        }
    }
}

mod tests {
    #[test]
    fn test_connection() {}
}
