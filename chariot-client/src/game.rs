use chariot_core::packets::ServerUpdatingPacket;
use std::net::TcpStream;

pub struct GameClient {
    connection: TcpStream,
}

impl GameClient {
    pub fn new(ip_addr: String) -> GameClient {
        let connection = TcpStream::connect(&ip_addr)
			.expect("could not connect to game server");
        // disable the Nagle algorithm to allow for real-time transfers
        connection
            .set_nodelay(true)
            .expect("could not turn off TCP delay");
        println!("game client now listening on {}", ip_addr);
        GameClient { connection }
    }

    pub fn ping(&self) {
        ServerUpdatingPacket::Ping
            .write_packet(&self.connection)
            .expect("failed to send ping packet")
    }
}
