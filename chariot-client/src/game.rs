use chariot_core::networking::{ServerBoundPacket, ServerConnection};
use chariot_core::player::player_inputs::InputEvent;
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

    pub fn _sync_outgoing(&mut self) {
        self.connection.sync_outgoing();
    }

    pub fn fetch_incoming_packets(&mut self) {
        self.connection.fetch_incoming_packets();
    }

    pub fn pick_chair(&mut self, chair: String) {
        self.connection
            .push_outgoing(ServerBoundPacket::ChairSelect(chair));
        self.connection.sync_outgoing();
    }

    pub fn pick_map(&mut self, map: String) {
        self.connection
            .push_outgoing(ServerBoundPacket::MapSelect(map));
        self.connection.sync_outgoing();
    }

    pub fn signal_ready_status(&mut self, ready: bool) {
        self.connection
            .push_outgoing(ServerBoundPacket::SetReadyStatus(ready));
        self.connection.sync_outgoing();
    }

    pub fn force_start(&mut self) {
        self.connection.push_outgoing(ServerBoundPacket::ForceStart);
        self.connection.sync_outgoing();
    }

    pub fn signal_loaded(&mut self) {
        self.connection
            .push_outgoing(ServerBoundPacket::NotifyLoaded);
        self.connection.sync_outgoing();
    }

    pub fn send_input_event(&mut self, event: InputEvent) {
        self.connection
            .push_outgoing(ServerBoundPacket::InputToggle(event));
        self.connection.sync_outgoing();
    }
}
