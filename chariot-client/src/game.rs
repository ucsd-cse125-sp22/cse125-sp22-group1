use chariot_core::networking::{ServerBoundPacket, ServerConnection};
use chariot_core::player_inputs::InputEvent;
use std::net::TcpStream;

pub struct GameClient {
    pub connection: ServerConnection,
}

impl GameClient {
    pub fn new(ip_addr: String) -> GameClient {
        let connection = TcpStream::connect(&ip_addr).expect("could not connect to game server");
        println!("game client now listening on {}", ip_addr);
        GameClient {
            connection: ServerConnection::new(connection),
        }
    }

    pub fn sync_outgoing(&mut self) {
        self.connection.sync_outgoing();
    }

    pub fn fetch_incoming_packets(&mut self) {
        self.connection.fetch_incoming_packets();
    }

    pub fn send_ready_packet(&mut self, chair_name: String) {
        self.connection
            .push_outgoing(ServerBoundPacket::ChairSelectAndReady(chair_name));
        self.connection.sync_outgoing();
    }

    pub fn send_input_event(&mut self, event: InputEvent) {
        self.connection
            .push_outgoing(ServerBoundPacket::InputToggle(event));
        self.connection.sync_outgoing();
    }
}
