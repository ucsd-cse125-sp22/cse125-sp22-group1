use crate::ui_state::CountdownState;
use chariot_core::player::choices::Chair;
use std::time::Duration;

// main menu
pub const HOME_BACKGROUND: &[u8] = include_bytes!("homebackground.png");
pub const MAIN_MENU_BACKGROUND: &[u8] = include_bytes!("main-menu/BG.png");
pub const MAIN_MENU_FOREGROUND: &[u8] = include_bytes!("main-menu/Chars.png");
pub const MAIN_MENU_TITLE: &[u8] = include_bytes!("main-menu/Title.png");

// icon
pub const ICON: &[u8] = include_bytes!("icon.png");

// chair select rectangles
pub const CHAIR_SELECT_BACKGROUND: &[u8] = include_bytes!("chair-select/background.png");
const CHAIR_SELECT_RECT0: &[u8] = include_bytes!("chair-select/select/p0rectangle.png");
const CHAIR_SELECT_RECT1: &[u8] = include_bytes!("chair-select/select/p1rectangle.png");
const CHAIR_SELECT_RECT2: &[u8] = include_bytes!("chair-select/select/p2rectangle.png");
const CHAIR_SELECT_RECT3: &[u8] = include_bytes!("chair-select/select/p3rectangle.png");
pub const CHAIR_SELECT_RECT: [&[u8]; 4] = [
    CHAIR_SELECT_RECT0,
    CHAIR_SELECT_RECT1,
    CHAIR_SELECT_RECT2,
    CHAIR_SELECT_RECT3,
];

// chair images
const CHAIR_BEANBAG: &[u8] = include_bytes!("chair-select/display/type=beanbag.png");
const CHAIR_ERGONOMIC: &[u8] = include_bytes!("chair-select/display/type=ergonomic.png");
const CHAIR_FOLDING: &[u8] = include_bytes!("chair-select/display/type=folding.png");
const CHAIR_RECLINER: &[u8] = include_bytes!("chair-select/display/type=recliner.png");
const CHAIR_SWIVEL: &[u8] = include_bytes!("chair-select/display/type=swivel.png");
const CHAIR_NONE: &[u8] = include_bytes!("chair-select/display/type=none.png");

pub fn get_chair_image(chair: Option<Chair>) -> &'static [u8] {
    match chair {
        Some(chair) => match chair {
            Chair::Swivel => CHAIR_SWIVEL,
            Chair::Recliner => CHAIR_RECLINER,
            Chair::Beanbag => CHAIR_BEANBAG,
            Chair::Ergonomic => CHAIR_ERGONOMIC,
            Chair::Folding => CHAIR_FOLDING,
        },
        None => CHAIR_NONE,
    }
}

// chair descriptions
const BEANBAG_DESCRIPTION: &[u8] = include_bytes!("chair-select/descriptions/beanbag.png");
const ERGONOMIC_DESCRIPTION: &[u8] = include_bytes!("chair-select/descriptions/ergonomic.png");
const FOLDING_DESCRIPTION: &[u8] = include_bytes!("chair-select/descriptions/folding.png");
const RECLINER_DESCRIPTION: &[u8] = include_bytes!("chair-select/descriptions/recliner.png");
const SWIVEL_DESCRIPTION: &[u8] = include_bytes!("chair-select/descriptions/swivel.png");

pub fn get_chair_description(chair: Chair) -> &'static [u8] {
    match chair {
        Chair::Swivel => SWIVEL_DESCRIPTION,
        Chair::Recliner => RECLINER_DESCRIPTION,
        Chair::Beanbag => BEANBAG_DESCRIPTION,
        Chair::Ergonomic => ERGONOMIC_DESCRIPTION,
        Chair::Folding => FOLDING_DESCRIPTION,
    }
}

// minimap
pub const TRACK_TRANSPARENT: &[u8] = include_bytes!("minimap/track_transparent.png");
const P1_BUTTON: &[u8] = include_bytes!("map-select/P1Btn.png");
const P2_BUTTON: &[u8] = include_bytes!("map-select/P2Btn.png");
const P3_BUTTON: &[u8] = include_bytes!("map-select/P3Btn.png");
const P4_BUTTON: &[u8] = include_bytes!("map-select/P4Btn.png");
pub const PLAYER_BUTTONS: [&[u8]; 4] = [P1_BUTTON, P2_BUTTON, P3_BUTTON, P4_BUTTON];

// placement
const FIRST_PLACE: &[u8] = include_bytes!("placement/1st.png");
const SECOND_PLACE: &[u8] = include_bytes!("placement/2nd.png");
const THIRD_PLACE: &[u8] = include_bytes!("placement/3rd.png");
const FOURTH_PLACE: &[u8] = include_bytes!("placement/4th.png");
pub const PLACE_IMAGES: [&[u8]; 4] = [FIRST_PLACE, SECOND_PLACE, THIRD_PLACE, FOURTH_PLACE];

// countdown
const COUNTDOWN_3: &[u8] = include_bytes!("countdown/3.png");
const COUNTDOWN_2: &[u8] = include_bytes!("countdown/2.png");
const COUNTDOWN_1: &[u8] = include_bytes!("countdown/1.png");
const COUNTDOWN_START: &[u8] = include_bytes!("countdown/start.png");

pub fn get_countdown_asset(countdown_state: CountdownState) -> Option<&'static [u8]> {
    match countdown_state {
        CountdownState::None => None,
        CountdownState::Three => Some(COUNTDOWN_3),
        CountdownState::Two => Some(COUNTDOWN_2),
        CountdownState::One => Some(COUNTDOWN_1),
        CountdownState::Start => Some(COUNTDOWN_START),
    }
}

// blank ui helper
pub const WHITE_TEXTURE: &[u8] = include_bytes!("box.png");
