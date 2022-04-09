use std::io::{Read, Write};

use bincode::{DefaultOptions, Options, Result};
use serde::{Deserialize, Serialize};

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

impl ServerUpdatingPacket {
    pub fn parse_packet<R: Read>(reader: R) -> Result<ServerUpdatingPacket> {
        DefaultOptions::new().deserialize_from(reader)
    }

    pub fn write_packet<W: Write>(&self, mut write: W) -> Result<()> {
        let options = DefaultOptions::new();
        let size = options.serialized_size(self)?;

        write.write_all(&[(size >> 8) as u8, size as u8])?;
        options.serialize_into(&mut write, self)
    }
}
