use std::io::{Read, Write};

use bincode::{DefaultOptions, Options, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::player_inputs::InputEvent;

#[derive(Serialize, Deserialize)]
pub enum ServerUpdatingPacket {
    Ping,
    InputToggle(InputEvent, bool),
}

#[derive(Serialize, Deserialize)]
pub enum ClientUpdatingPacket {
    Pong,
    //GameStateUpdate(GameState),
}

pub trait Packet: Serialize + DeserializeOwned {
    fn parse_packet<R: Read>(reader: &mut R) -> Result<Self> {
        DefaultOptions::new().deserialize_from(reader)
    }
    fn packet_size(&self) -> Result<u64> {
        DefaultOptions::new().serialized_size(self)
    }
    fn write_packet<W: Write>(&self, write: &mut W) -> Result<()> {
        DefaultOptions::new().serialize_into(write, self)
    }
}

impl Packet for ClientUpdatingPacket {}
impl Packet for ServerUpdatingPacket {}
