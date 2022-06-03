use std::io::{Read, Write};
use std::time::Duration;

use bincode::{DefaultOptions, Options, Result};
use glam::DVec3;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub use uuid::Uuid;

use crate::entity_location::EntityLocation;
use crate::player::choices::{Chair, PlayerChoices, Track};
use crate::player::{
    lap_info::{LapNumber, Placement},
    player_inputs::InputEvent,
    PlayerID,
};
use crate::questions::{QuestionData, QuestionOption};
use crate::sound_effect::SoundEffect;

#[derive(Serialize, Deserialize)]
pub enum ServerBoundPacket {
    // Before game
    ChairSelect(Chair),
    MapSelect(Track),
    SetReadyStatus(bool),
    ForceStart,
    NotifyLoaded,

    // During game
    InputToggle(InputEvent),

    // After game
    NextGame,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ClientBoundPacket {
    // Before game
    PlayerNumber(PlayerID, [Option<PlayerChoices>; 4]),
    PlayerChairChoice(PlayerID, Chair), // Another player has hovered a chair
    PlayerMapChoice(PlayerID, Track),   // Another player has hovered a map
    PlayerReadyStatus(PlayerID, bool),  // Another player has readied or unreaded
    PlayerJoined(PlayerID),

    // Load into the game
    LoadGame(Track), // Map name, each player's chair

    // Pre-game
    GameStart(Duration), // How long until the game starts?

    // During game
    EntityUpdate(Vec<(EntityLocation, DVec3, bool)>), // Clients will need to know the location and velocity of every player
    PowerupPickup,                                    // Add a payload here when appropriate
    VotingStarted {
        question: QuestionData,
        #[serde(with = "serde_millis")]
        time_until_vote_end: Duration,
    }, // Sent when the audience begins voting (suspense!)
    VotingUpdate(Vec<u32>),
    InteractionActivate {
        question: QuestionData,
        decision: QuestionOption,
        #[serde(with = "serde_millis")]
        time_effect_is_live: Duration,
        winner_idx: usize,
    }, // Sent when the audience has voted on something
    VotingCooldown,
    LapUpdate(LapNumber),       // What lap are you now on?
    PlacementUpdate(Placement), // What place in the race are you now at?
    FinishedLaps(Placement), // You completed all laps, what place are you?

    SoundEffectEvent(SoundEffect),

    // After game
    AllDone {
        // [place, time: (seconds, nanoseconds)]
        placements: [(Placement, (u64, u32)); 4],
    },
    StartNextGame,
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
