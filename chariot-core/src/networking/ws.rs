use tungstenite::{accept, Message, WebSocket};

use crate::networking::Packet;
use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;

pub struct WSConnection {
    socket: WebSocket<TcpStream>,
    incoming_packets: VecDeque<Message>,
    outgoing_packets: VecDeque<Message>,
}

impl WSConnection {
    pub fn new(tcp_stream: TcpStream) -> WSConnection {
        match accept(tcp_stream) {
            Ok(socket) => {
                return WSConnection {
                    socket,
                    incoming_packets: VecDeque::new(),
                    outgoing_packets: VecDeque::new(),
                }
            }
            Err(err) => {
                println!("{:?}", err);
                panic!("problem!");
            }
        }
    }

    fn set_nonblocking(&self) -> () {
        // self.tcp_stream
        //     .set_nonblocking(true)
        //     .expect("failed to set connection as non-blocking");
    }

    fn set_blocking(&self) -> () {
        // self.tcp_stream
        //     .set_nonblocking(false)
        //     .expect("failed to set connection back to blocking");
    }

    pub fn sync_incoming(&mut self) {
        let msg = self
            .socket
            .read_message()
            .expect("should be able to read something");
        if msg.is_binary() || msg.is_text() {
            // this is where we handle shit
            self.incoming_packets.push_back(msg);
        }
        // // fetch packets for this connection until exhausted
        // loop {
        //     // allows us to keep going if there's no input
        //     self.set_nonblocking();

        //     // attempt to parse the two bytes at the beginning of each well-formed packet
        //     // that represents the size in bytes of the incoming payload
        //     let mut buffer: [u8; 2] = [0, 0];
        //     let packet_size = match self.tcp_stream.read_exact(&mut buffer) {
        //         Ok(_) => ((buffer[0] as u16) << 8) | buffer[1] as u16,
        //         // this error just means there's not enough new client data on this connection
        //         Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
        //         // this error means one of our clients disconnected
        //         // TODO: handle by removing this connection from the client pool
        //         Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
        //             break;
        //         }
        //         // anything else is unexpected, so fail fast and hard
        //         Err(e) => panic!(
        //             "encountered unfamiliar IO error while polling client events: {:?}",
        //             e
        //         ),
        //     };

        //     // if we parsed a packet size, let's go ahead and read that amount,
        //     // this time blocking until we've parsed the entire thing
        //     self.set_blocking();
        //     let packet =
        //         T::parse_packet(&mut Read::by_ref(&mut self.tcp_stream).take(packet_size as u64))
        //             .expect("Failed to deserialize packet");

        //     self.incoming_packets.push_back(packet);
        // }
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
