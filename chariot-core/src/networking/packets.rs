use std::io::{Read, Write};

use bincode::{DefaultOptions, Options, Result};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

#[derive(Serialize, Deserialize)]
pub enum ServerUpdatingPacket {
    Ping,
    //InputToggle(Input, bool),
}

#[derive(Serialize, Deserialize)]
pub enum ClientUpdatingPacket {
    Pong,
    //GameStateUpdate(GameState),
}

pub trait Packet: Serialize + DeserializeOwned {
    fn parse_packet<R: Read>(reader: R) -> Result<Self> {
        DefaultOptions::new().deserialize_from(reader)
    }
    fn write_packet<W: Write>(&self, mut write: W) -> Result<()> {
        let options = DefaultOptions::new();
        let size = options.serialized_size(self)?;

        write.write_all(&[(size >> 8) as u8, size as u8])?;
        options.serialize_into(&mut write, self)
    }
}

impl Packet for ClientUpdatingPacket {}
impl Packet for ServerUpdatingPacket {}