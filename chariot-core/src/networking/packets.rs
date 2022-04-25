use std::io::{Read, Write};

use bincode::{DefaultOptions, Options, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::entity_location::EntityLocation;
use crate::player_inputs::InputEvent;

#[derive(Serialize, Deserialize)]
pub enum ServerBoundPacket {
    // Debug
    Ping,

    // Before game
    ChairSelectAndReady(String), // name of chair being selected

    // During game
    InputToggle(InputEvent),
}

#[derive(Serialize, Deserialize)]
pub enum ClientBoundPacket {
    // Debug
    Pong,
    Message(String),

    // Before game
    PlayerNumber(u8),
    EveryoneReady,

    // During game
    LocationUpdate(EntityLocation),
    PowerupPickup,       // Add a payload here when appropriate
    InteractionActivate, // Add a payload here when appropriate
    LapUpdate(u8),       // What lap are you now on?
    PlacementUpdate(u8), // What place in the race are you now at?

    // After game
    AllDone,
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

impl Packet for ClientBoundPacket {}
impl Packet for ServerBoundPacket {}
