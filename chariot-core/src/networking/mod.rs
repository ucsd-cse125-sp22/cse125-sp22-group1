mod connection;
mod packets;
pub mod ws;

pub use packets::*;
pub type ClientConnection = connection::Connection<ServerUpdatingPacket, ClientUpdatingPacket>;
pub type ServerConnection = connection::Connection<ClientUpdatingPacket, ServerUpdatingPacket>;
pub type WebSocketConnection = ws::WSConnection;
