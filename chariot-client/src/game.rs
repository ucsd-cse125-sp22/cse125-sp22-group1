use crate::client_events::Watching;
use chariot_core::networking::{ClientUpdatingPacket, ServerConnection, ServerUpdatingPacket};
use std::collections::HashSet;
use std::net::TcpStream;
use winit::event::{ElementState, VirtualKeyCode};

pub struct GameClient {
    connection: ServerConnection,
    pressed_keys: HashSet<VirtualKeyCode>,
}

impl Watching for GameClient {
    fn on_key_down(&mut self, key: VirtualKeyCode) {
        println!("Key down [{:?}]!", key);
        self.pressed_keys.insert(key);
        // Call self.some_other_object.on_key_down(key); to pass it along
    }

    fn on_key_up(&mut self, key: VirtualKeyCode) {
        println!("Key up [{:?}]!", key);
        self.pressed_keys.remove(&key);
    }

    fn on_mouse_move(&mut self, x: f64, y: f64) {
        println!("Mouse moved! ({}, {})", x, y);
    }

    fn on_left_mouse(&mut self, x: f64, y: f64, state: ElementState) {
        if let ElementState::Released = state {
            println!("Mouse clicked @ ({}, {})!", x, y);
        }
    }

    fn on_right_mouse(&mut self, x: f64, y: f64, state: ElementState) {
        if let ElementState::Released = state {
            println!("Mouse right clicked @ ({}, {})!", x, y);
        }
    }
}

impl GameClient {
    pub fn new(ip_addr: String) -> GameClient {
        let connection = TcpStream::connect(&ip_addr).expect("could not connect to game server");
        println!("game client now listening on {}", ip_addr);
        GameClient {
            connection: ServerConnection::new(connection),
            pressed_keys: HashSet::new(),
        }
    }

    pub fn print_keys(&self) {
        println!("Pressed keys: {:?}", self.pressed_keys)
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
