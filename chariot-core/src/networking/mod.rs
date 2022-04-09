mod connection;
mod packets;

pub use packets::*;
pub type ClientConnection = connection::Connection<ServerUpdatingPacket,ClientUpdatingPacket>;
pub type ServerConnection = connection::Connection<ClientUpdatingPacket,ServerUpdatingPacket>;