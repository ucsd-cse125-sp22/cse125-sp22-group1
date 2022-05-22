use chariot_core::networking::{ServerBoundPacket, ServerConnection};
use chariot_core::player::choices::{Chair, Track};
use chariot_core::player::player_inputs::InputEvent;
use std::net::TcpStream;
use std::time::Instant;

pub enum AnnouncementState {
    None,
    GeneralAnnouncement {
        title: String,
        subtitle: String,
    },
    VotingInProgress {
        prompt: String,
        vote_end_time: Instant,
    },
    VoteActiveTime {
        prompt: String,
        decision: String,
        effect_end_time: Instant,
    },
}

pub struct GameClient {
    pub connection: ServerConnection,
    pub announcement_state: AnnouncementState,
}

impl GameClient {
    pub fn new(ip_addr: String) -> GameClient {
        let connection = TcpStream::connect(&ip_addr).expect("could not connect to game server");
        println!("game client now listening on {}", ip_addr);
        GameClient {
            connection: ServerConnection::new(connection),
            announcement_state: AnnouncementState::None,
        }
    }

    pub fn fetch_incoming_packets(&mut self) {
        self.connection.fetch_incoming_packets();
    }

    fn send_packet(&mut self, packet: ServerBoundPacket) {
        self.connection.push_outgoing(packet);
        self.connection.sync_outgoing();
    }

    pub fn pick_chair(&mut self, chair: Chair) {
        self.send_packet(ServerBoundPacket::ChairSelect(chair));
    }

    pub fn pick_map(&mut self, map: Track) {
        self.send_packet(ServerBoundPacket::MapSelect(map));
    }

    pub fn signal_ready_status(&mut self, ready: bool) {
        self.send_packet(ServerBoundPacket::SetReadyStatus(ready));
    }

    pub fn force_start(&mut self) {
        self.send_packet(ServerBoundPacket::ForceStart);
    }

    pub fn signal_loaded(&mut self) {
        self.send_packet(ServerBoundPacket::NotifyLoaded);
    }

    pub fn send_input_event(&mut self, event: InputEvent) {
        self.send_packet(ServerBoundPacket::InputToggle(event));
    }

    pub fn next_game(&mut self) {
        self.send_packet(ServerBoundPacket::NextGame);
    }
}
