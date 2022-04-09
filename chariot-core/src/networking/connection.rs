use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;

use super::Packet;

pub struct Connection<T: Packet, V: Packet> {
    tcp_stream: TcpStream,
    incoming_packets: VecDeque<T>,
    outgoing_packets: VecDeque<V>,
}

impl<T: Packet, V: Packet> Connection<T, V> {
    pub fn new(tcp_stream: TcpStream) -> Connection<T, V> {
        // disable the Nagle algorithm to allow for real-time transfers
        tcp_stream
            .set_nodelay(true)
            .expect("could not turn off TCP delay");
        Connection {
            tcp_stream,
            incoming_packets: VecDeque::new(),
            outgoing_packets: VecDeque::new(),
        }
    }

    fn set_nonblocking(&self) -> () {
        self.tcp_stream
            .set_nonblocking(true)
            .expect("failed to set connection as non-blocking");
    }

    fn set_blocking(&self) -> () {
        self.tcp_stream
            .set_nonblocking(false)
            .expect("failed to set connection back to blocking");
    }

    pub fn sync_incoming(&mut self) {
        // fetch packets for this connection until exhausted
        loop {
            // allows us to keep going if there's no input
            self.set_nonblocking();

            // attempt to parse the two bytes at the beginning of each well-formed packet
            // that represents the size in bytes of the incoming payload
            let mut buffer: [u8; 2] = [0, 0];
            let packet_size = match self.tcp_stream.read_exact(&mut buffer) {
                Ok(_) => ((buffer[0] as u16) << 8) | buffer[1] as u16,
                // this error just means there's not enough new client data on this connection
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                // this error means one of our clients disconnected
                // TODO: handle by removing this connection from the client pool
                Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
                    break;
                }
                // anything else is unexpected, so fail fast and hard
                Err(e) => panic!(
                    "encountered unfamiliar IO error while polling client events: {:?}",
                    e
                ),
            };

            // if we parsed a packet size, let's go ahead and read that amount,
            // this time blocking until we've parsed the entire thing
            self.set_blocking();
            let packet = T::parse_packet(&mut Read::by_ref(&mut self.tcp_stream).take(packet_size as u64))
                .expect("Failed to deserialize packet");

            self.incoming_packets.push_back(packet);
        }
    }

    pub fn pop_incoming(&mut self) -> Option<T> {
        self.incoming_packets.pop_front()
    }

    pub fn push_outgoing(&mut self, packet: V) -> () {
        self.outgoing_packets.push_back(packet);
    }

    // send packets on this connection until exhausted
    pub fn sync_outgoing(&mut self) {
        while let Some(packet) = self.outgoing_packets.pop_front() {
            let size = packet.packet_size()
                .expect("failed to get serialize packet size");
            self.tcp_stream.write_all(&[(size >> 8) as u8, size as u8])
                .expect("failed to write packet size");
            packet.write_packet(&mut self.tcp_stream)
                .expect("failed to write packet");
        }
    }
}
