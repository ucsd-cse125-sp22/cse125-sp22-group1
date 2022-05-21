pub fn get_minimap_player_location(location: (f32, f32)) -> (f32, f32) {
    // these values are guesses btw
    const MIN_TRACK_X: f32 = -113.0;
    const MAX_TRACK_X: f32 = 37.0;
    const MIN_TRACK_Z: f32 = -39.0;
    const MAX_TRACK_Z: f32 = 111.0;

    (
        (MAX_TRACK_Z - location.1) / (MAX_TRACK_Z - MIN_TRACK_Z),
        (location.0 - MIN_TRACK_X) / (MAX_TRACK_X - MIN_TRACK_X),
    )
}
