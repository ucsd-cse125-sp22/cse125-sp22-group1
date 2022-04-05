use std::{io, thread};
use std::collections::VecDeque;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

use chariot_core::GLOBAL_CONFIG;
use chariot_core::packets::ServerUpdatingPacket;

pub struct GameServer {
    listener: TcpListener,
    connections: Vec<TcpStream>,
    incoming_packets: VecDeque<ServerUpdatingPacket>,
    // outgoing_packets: VecDeque<ClientUpdatingPacket>
}

impl GameServer {
    pub fn new(ip_addr: String) -> GameServer {
        // start the TCP listening service
        let listener = TcpListener::bind(&ip_addr)
            .expect("could not bind to configured server address");
        println!("game server now listening on {}", ip_addr);
        let connections: Vec<TcpStream> = Vec::new();
        let incoming_packets : VecDeque<ServerUpdatingPacket> = VecDeque::new();
        GameServer { listener, connections, incoming_packets }
    }

    pub fn start_loop(&mut self) {
        let max_server_tick_duration = Duration::from_millis(GLOBAL_CONFIG.server_tick_ms);

        loop {
            self.block_until_minimum_connections();

            let start_time = Instant::now();

            self.fetch_client_events();
            self.process_incoming_packets();
            self.simulate_game();
            self.send_state_and_events();

            // wait until server tick time has elapsed
            let remaining_tick_duration = max_server_tick_duration.checked_sub(start_time.elapsed())
                .expect("server tick took longer than configured length");
            thread::sleep(remaining_tick_duration);
        }
    }

    // blocks the primary loop if we don't have the minimum players
    fn block_until_minimum_connections(&mut self) {
        while self.connections.len() < GLOBAL_CONFIG.player_amount {
            match self.listener.accept() {
                Ok((socket, addr)) => {
                    println!("new connection from {}", addr.ip().to_string());
                    // disable the Nagle algorithm to allow for real-time transfers
                    socket.set_nodelay(true)
                        .expect("could not turn off TCP delay");
                    self.connections.push(socket);
                }
                Err(e) => println!("couldn't get connecting client info {:?}", e)
            }
        }
    }

    // poll for input events and add them to the incoming packet queue
    fn fetch_client_events(&mut self) {
        for mut connection in self.connections.iter() {
            // fetch packets for this connection until exhausted
            loop {
                // allows us to keep going if there's no input
                connection.set_nonblocking(true)
                    .expect("failed to set connection as non-blocking");

                // attempt to parse the two bytes at the beginning of each well-formed packet
                // that represents the size in bytes of the incoming payload
                let mut buffer: [u8;2] = [0,0];
                let packet_size = match connection.read_exact(&mut buffer) {
                    Ok(_) => ((buffer[0] as u16) << 8) | buffer[1] as u16,
                    // this error just means there's not enough new client data on this connection
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    // this error means one of our clients disconnected
                    // TODO: handle by removing this connection from the client pool
                    Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => { break; },
                    // anything else is unexpected, so fail fast and hard
                    Err(e) => panic!("encountered unfamiliar IO error while polling client events: {:?}", e),
                };

                // if we parsed a packet size, let's go ahead and read that amount,
                // this time blocking until we've parsed the entire thing
                connection.set_nonblocking(false)
                    .expect("failed to set connection back to blocking");
                let packet = ServerUpdatingPacket::parse_packet(connection.take(packet_size as u64))
                    .expect("Failed to deserialize packet");

                self.incoming_packets.push_back(packet);
            }
        }
    }

    // handle every packet in received order
    fn process_incoming_packets(&mut self) {
        while let Some(packet) = self.incoming_packets.pop_front() {
            match packet {
                ServerUpdatingPacket::Ping => println!("Received a Ping packet!")
            }
        }
    }

    // update game state
    fn simulate_game(&mut self) {

    }

    // send out updated game state and other events
    fn send_state_and_events(&mut self) {

    }
}