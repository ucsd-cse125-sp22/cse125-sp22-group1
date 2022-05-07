use std::io::{Read, Write};

use bincode::{DefaultOptions, Options, Result};
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
    EveryoneReady,

    // During game
    LocationUpdate([Option<EntityLocation>; 4]), // Clients will need to know the location of every player
    PowerupPickup,                               // Add a payload here when appropriate
    InteractionActivate,                         // Add a payload here when appropriate
    LapUpdate(u8),                               // What lap are you now on?
    PlacementUpdate(u8),                         // What place in the race are you now at?

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

#[derive(Serialize, Deserialize)]
pub enum WSAudienceBoundMessage {
    Prompt(String, (String, String, String, String)), // Question, 4 Answer Choices

    Winner(u32), // The winning choice (tuple index)

    Assignment(Uuid), // Sends a uuid that the server will use to identify the client
}

#[derive(Serialize, Deserialize)]
pub enum WSServerBoundMessage {
    Vote(String, u32), // Client UUID, the option to vote for
}
