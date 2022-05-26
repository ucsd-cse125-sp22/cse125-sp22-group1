use chariot_core::player::choices::Chair;

// backgrounds
pub const HOME_BACKGROUND: &[u8] = include_bytes!("homebackground.png");
pub const CHAIR_SELECT_BACKGROUND: &[u8] = include_bytes!("chair-select/background.png");

// chair select rectangles
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

pub fn get_chair_image(chair: Chair) -> &'static [u8] {
    match chair {
        Chair::Swivel => CHAIR_SWIVEL,
        Chair::Recliner => CHAIR_RECLINER,
        Chair::Beanbag => CHAIR_BEANBAG,
        Chair::Ergonomic => CHAIR_ERGONOMIC,
        Chair::Folding => CHAIR_FOLDING,
    }
}

// minimap
pub const TRACK_TRANSPARENT: &[u8] = include_bytes!("minimap/track_transparent.png");
const P1_BUTTON: &[u8] = include_bytes!("map-select/P1Btn.png");
const P2_BUTTON: &[u8] = include_bytes!("map-select/P2Btn.png");
const P3_BUTTON: &[u8] = include_bytes!("map-select/P3Btn.png");
const P4_BUTTON: &[u8] = include_bytes!("map-select/P4Btn.png");
pub const PLAYER_BUTTONS: [&[u8]; 4] = [P1_BUTTON, P2_BUTTON, P3_BUTTON, P4_BUTTON];
