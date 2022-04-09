use chariot_core::networking::{ClientUpdatingPacket, ServerConnection, ServerUpdatingPacket};
use std::net::TcpStream;

pub struct GameClient {
    connection: ServerConnection,
}

impl GameClient {
    pub fn new(ip_addr: String) -> GameClient {
        let connection = TcpStream::connect(&ip_addr).expect("could not connect to game server");
        println!("game client now listening on {}", ip_addr);
        GameClient {
            connection: ServerConnection::new(connection),
        }
    }

    pub fn ping(&mut self) {
        self.connection.push_outgoing(ServerUpdatingPacket::Ping);
    }

    pub fn sync_outgoing(&mut self) {
        self.connection.sync_outgoing();
    }

    pub fn sync_incoming(&mut self) {
        self.connection.sync_incoming();
    }

    pub fn process_incoming_packets(&mut self) {
        while let Some(packet) = self.connection.pop_incoming() {
            match packet {
                ClientUpdatingPacket::Pong => {
                    println!("Received a Pong packet from server!");
                }
            }
        }
    }
}
