use std::io::{Read, Write};
use std::time::Duration;

use bincode::{DefaultOptions, Options, Result};
use glam::DVec3;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub use uuid::Uuid;

use crate::entity_location::EntityLocation;
use crate::player::choices::PlayerChoices;
use crate::player::{
    lap_info::{LapNumber, Placement},
    player_inputs::InputEvent,
    PlayerID,
};
use crate::questions::{QuestionData, QuestionOption};

#[derive(Serialize, Deserialize)]
pub enum ServerBoundPacket {
    // Before game
    ChairSelect(String),
    MapSelect(String),
    SetReadyStatus(bool),
    ForceStart,
    NotifyLoaded,

    // During game
    InputToggle(InputEvent),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ClientBoundPacket {
    // Before game
    PlayerNumber(PlayerID, [Option<PlayerChoices>; 4]),
    PlayerChairChoice(PlayerID, String), // Another player has hovered a chair
    PlayerMapChoice(PlayerID, String),   // Another player has hovered a map
    PlayerReadyStatus(PlayerID, bool),   // Another player has readied or unreaded
    PlayerJoined(PlayerID),

    // Load into the game
    LoadGame(String), // Map name, each player's chair

    // Pre-game
    GameStart(Duration), // How long until the game starts?

    // During game
    EntityUpdate(Vec<(EntityLocation, DVec3)>), // Clients will need to know the location and velocity of every player
    PowerupPickup,                              // Add a payload here when appropriate
    VotingStarted(QuestionData),                // Sent when the audience begins voting (suspense!)
    InteractionActivate(QuestionData, QuestionOption), // Sent when the audience has voted on something
    LapUpdate(LapNumber),                              // What lap are you now on?
    PlacementUpdate(Placement),                        // What place in the race are you now at?

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
