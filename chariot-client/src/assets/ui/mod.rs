use crate::ui_state::CountdownState;
use chariot_core::player::choices::Chair;
use glam::Vec2;

// main menu
pub const HOME_BACKGROUND: &[u8] = include_bytes!("homebackground.png");
pub const _MAIN_MENU_BACKGROUND: &[u8] = include_bytes!("main-menu/BG.png");
pub const _MAIN_MENU_FOREGROUND: &[u8] = include_bytes!("main-menu/Chars.png");
pub const _MAIN_MENU_TITLE: &[u8] = include_bytes!("main-menu/Title.png");

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

pub const RESULTS_BACKGROUND: &[u8] = include_bytes!("final-standings/results-background.png");

// final standings placement cards
const PLAYER_1_PLACEMENT_CARD: &[u8] = include_bytes!("final-standings/results-1.png");
const PLAYER_2_PLACEMENT_CARD: &[u8] = include_bytes!("final-standings/results-2.png");
const PLAYER_3_PLACEMENT_CARD: &[u8] = include_bytes!("final-standings/results-3.png");
const PLAYER_4_PLACEMENT_CARD: &[u8] = include_bytes!("final-standings/results-4.png");
pub const PLACEMENT_CARDS: [&[u8]; 4] = [
    PLAYER_1_PLACEMENT_CARD,
    PLAYER_2_PLACEMENT_CARD,
    PLAYER_3_PLACEMENT_CARD,
    PLAYER_4_PLACEMENT_CARD,
];

// final standings cropped chair images
const SWIVEL_CHAIR_ICON: &[u8] = include_bytes!("final-standings/swivel-circle.png");
const RECLINER_CHAIR_ICON: &[u8] = include_bytes!("final-standings/reclining-circle.png");
const BEANBAG_CHAIR_ICON: &[u8] = include_bytes!("final-standings/beanbag-circle.png");
const ERGONOMIC_CHAIR_ICON: &[u8] = include_bytes!("final-standings/ergonomic-circle.png");
const FOLDING_CHAIR_ICON: &[u8] = include_bytes!("final-standings/folding-circle.png");

pub fn get_chair_icon(chair: Chair) -> &'static [u8] {
    match chair {
        Chair::Swivel => SWIVEL_CHAIR_ICON,
        Chair::Recliner => RECLINER_CHAIR_ICON,
        Chair::Beanbag => BEANBAG_CHAIR_ICON,
        Chair::Ergonomic => ERGONOMIC_CHAIR_ICON,
        Chair::Folding => FOLDING_CHAIR_ICON,
    }
}
// countdown
const COUNTDOWN_3: &[u8] = include_bytes!("countdown/3.png");
const COUNTDOWN_2: &[u8] = include_bytes!("countdown/2.png");
const COUNTDOWN_1: &[u8] = include_bytes!("countdown/1.png");
const COUNTDOWN_START: &[u8] = include_bytes!("countdown/start.png");

// join the audience
pub const JOIN_THE_AUDIENCE: &[u8] = include_bytes!("jointheaudience.png");

// return the asset AND DIMENSIONS of each asset
pub fn get_countdown_asset(countdown_state: CountdownState) -> Option<(&'static [u8], Vec2)> {
    match countdown_state {
        CountdownState::None => None,
        CountdownState::Three => Some((COUNTDOWN_3, Vec2::new(220.0, 220.0))),
        CountdownState::Two => Some((COUNTDOWN_2, Vec2::new(220.0, 220.0))),
        CountdownState::One => Some((COUNTDOWN_1, Vec2::new(220.0, 220.0))),
        CountdownState::Start => Some((COUNTDOWN_START, Vec2::new(920.0, 170.0))),
    }
}

// blank ui helper
pub const WHITE_TEXTURE: &[u8] = include_bytes!("box.png");
