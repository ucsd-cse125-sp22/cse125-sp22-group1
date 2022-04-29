use chariot_core::networking::{ClientBoundPacket, ServerBoundPacket, ServerConnection};
use chariot_core::player_inputs::InputEvent;
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
        self.connection.push_outgoing(ServerBoundPacket::Ping);
    }

    pub fn sync_outgoing(&mut self) {
        self.connection.sync_outgoing();
    }

    pub fn sync_incoming(&mut self) {
        self.connection.sync_incoming();
    }

    pub fn get_incoming_packets(&mut self) -> Vec<ClientBoundPacket> {
        let mut ret = vec![];
        while let Some(packet) = self.connection.pop_incoming() {
            ret.push(packet);
        }
        return ret;
    }

    pub fn send_ready_packet(&mut self, chair_name: String) {
        self.connection.push_outgoing(ServerBoundPacket::ChairSelectAndReady(chair_name));
        self.connection.sync_outgoing();
    }

    pub fn send_input_event(&mut self, event: InputEvent) {
        println!("sending input event");
        self.connection
            .push_outgoing(ServerBoundPacket::InputToggle(event));
        self.connection.sync_outgoing();
    }
}
