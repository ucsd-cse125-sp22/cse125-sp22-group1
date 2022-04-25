use chariot_core::networking::{ClientUpdatingPacket, ServerConnection, ServerUpdatingPacket};
use chariot_core::player_inputs::InputEvent;
use std::net::TcpStream;
use winit::event::{ElementState, VirtualKeyCode};

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
                ClientUpdatingPacket::Message(text) => {
                    println!("Recieved a message from the server saying: {}", text);
                }
            }
        }
    }

    pub fn send_input_event(&mut self, event: InputEvent) {
        self.connection
            .push_outgoing(ServerUpdatingPacket::InputToggle(event));
        self.connection.sync_outgoing();
    }
}
