mod connection;
mod packets;
pub mod ws;

pub use packets::*;
pub type ClientConnection = connection::Connection<ServerBoundPacket, ClientBoundPacket>;
pub type ServerConnection = connection::Connection<ClientBoundPacket, ServerBoundPacket>;
pub type WebSocketConnection = ws::WSConnection;
