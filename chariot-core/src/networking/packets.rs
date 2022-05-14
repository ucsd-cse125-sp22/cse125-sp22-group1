use std::io::{Read, Write};
use std::time::Duration;

use bincode::{DefaultOptions, Options, Result};
use glam::DVec3;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub use uuid::Uuid;

use crate::entity_location::EntityLocation;
use crate::player_inputs::InputEvent;

#[derive(Serialize, Deserialize)]
pub enum ServerBoundPacket {
    // Before game
    ChairSelectAndReady(String), // name of chair being selected

    // During game
    InputToggle(InputEvent),
}

#[derive(Serialize, Deserialize)]
pub enum ClientBoundPacket {
    // Before game
    PlayerNumber(u8),
    GameStart(Duration), // How long until the game starts?

    // During game
    EntityUpdate(Vec<(EntityLocation, DVec3)>), // Clients will need to know the location and velocity of every player
    PowerupPickup,                              // Add a payload here when appropriate
    InteractionActivate,                        // Add a payload here when appropriate
    LapUpdate(u8),                              // What lap are you now on?
    PlacementUpdate(u8),                        // What place in the race are you now at?

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
