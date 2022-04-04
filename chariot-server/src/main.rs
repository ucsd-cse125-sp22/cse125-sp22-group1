use std::net::TcpListener;
use chariot_core::GLOBAL_CONFIG;

fn main() {
    // start the TCP listening service
    //let listener = TcpListener::bind(&GLOBAL_CONFIG.server_address);
    println!("Hello, world!, server address {:?}", &GLOBAL_CONFIG.server_address);
}
