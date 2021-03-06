use chariot_core::sound_effect::SoundEffect;
use rand::prelude::SliceRandom;

// music
pub const HOLD_ON_TO_YOUR_SEATS: &[u8] = include_bytes!("music/01.ogg");
pub const CYBER_RECLINER: &[u8] = include_bytes!("music/04.ogg");
pub const TURBOBOOSTING_ALL_THE_WAY_HOME: &[u8] = include_bytes!("music/06.ogg");
pub const TRACK_MUSIC: [&[u8]; 2] = [HOLD_ON_TO_YOUR_SEATS, TURBOBOOSTING_ALL_THE_WAY_HOME];

pub fn random_track_music() -> &'static [u8] {
    *TRACK_MUSIC.choose(&mut rand::thread_rng()).unwrap()
}

// ambient

// sfx

const ENTER_CHAIRACTER_SELECT: &[u8] = include_bytes!("sfx/Enter_Chairacter_Select.ogg");
const SELECT_CHAIRACTER: &[u8] = include_bytes!("sfx/Select_Chairacter.ogg");
const READY_UP: &[u8] = include_bytes!("sfx/Ready_Up.ogg");

const GAME_START: &[u8] = include_bytes!("sfx/Success_01.ogg");
const NEXT_LAP: &[u8] = include_bytes!("sfx/Effect_02.ogg");
const GAME_END: &[u8] = include_bytes!("sfx/Success_01.ogg");

const PLAYER_COLLISION: &[u8] = include_bytes!("sfx/Bump_01.ogg");
const TERRAIN_COLLISION: &[u8] = include_bytes!("sfx/Bump_02.ogg");

const INTERACTION_VOTE_START: &[u8] = include_bytes!("sfx/Power_Get_01.ogg");
const INTERACTION_CHOSEN: &[u8] = include_bytes!("sfx/Effect_01.ogg");

const COUNTDOWN_GENERAL: &[u8] = include_bytes!("sfx/Countdown_Tick.ogg");
const COUNTDOWN_GO: &[u8] = include_bytes!("sfx/Countdown_Go.ogg");

pub fn get_sfx(effect: SoundEffect) -> &'static [u8] {
    match effect {
        SoundEffect::EnterChairacterSelect => ENTER_CHAIRACTER_SELECT,
        SoundEffect::SelectChairacter => SELECT_CHAIRACTER,
        SoundEffect::ReadyUp => READY_UP,
        SoundEffect::GameStart => GAME_START,
        SoundEffect::NextLap => NEXT_LAP,
        SoundEffect::GameEnd => GAME_END,
        SoundEffect::PlayerCollision => PLAYER_COLLISION,
        SoundEffect::TerrainCollision => TERRAIN_COLLISION,
        SoundEffect::InteractionVoteStart => INTERACTION_VOTE_START,
        SoundEffect::InteractionChosen => INTERACTION_CHOSEN,
        SoundEffect::CountdownGeneral => COUNTDOWN_GENERAL,
        SoundEffect::CountdownGo => COUNTDOWN_GO,
    }
}
