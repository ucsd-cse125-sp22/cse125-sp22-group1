use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};
use chariot_core::GLOBAL_CONFIG;

pub struct GameServer {
    listener: TcpListener,
    connections: Vec<TcpStream>
}

impl GameServer {
    pub fn new(ip_addr: String) -> GameServer {
        // start the TCP listening service
        let listener = TcpListener::bind(&ip_addr)
            .expect("could not bind to configured server address");
        println!("game server now listening on {}", ip_addr);
        let connections: Vec<TcpStream> = Vec::new();
        GameServer { listener, connections }
    }

    pub fn start_loop(&mut self) {
        let max_server_tick_duration = Duration::from_millis(GLOBAL_CONFIG.server_tick_ms);

        loop {
            // don't actually loop if we don't have the minimum players
            self.block_until_minimum_connections();

            let start_time = Instant::now();

            self.fetch_client_events();
            self.simulate_game();
            self.send_state_and_events();

            // wait until server tick time has elapsed
            let remaining_tick_duration = max_server_tick_duration.checked_sub(start_time.elapsed())
                .expect("server tick took longer than configured length");
            thread::sleep(remaining_tick_duration);
        }
    }

    fn block_until_minimum_connections(&mut self) {
        while self.connections.len() < GLOBAL_CONFIG.player_amount {
            match self.listener.accept() {
                Ok((socket, addr)) => {
                    println!("new connection from {}", addr.ip().to_string());
                    self.connections.push(socket);
                }
                Err(e) => println!("couldn't get connecting client info {:?}", e)
            }
        }
    }

    // poll for input events
    fn fetch_client_events(&mut self) {

    }

    // update game state
    fn simulate_game(&mut self) {

    }

    // send out updated game state and other events
    fn send_state_and_events(&mut self) {

    }
}