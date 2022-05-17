use image::{ImageBuffer, Rgb};

use super::ResourceManager;

pub fn create_minimap_image(
    player_locations: Vec<(f32, f32)>,
    resources: &mut ResourceManager,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut base_map = resources.get_minimap_image("track_transparent.png");
    let map_width = base_map.width();
    let map_height = base_map.height();

    // Place a dot at each player's location
    for (player_index, location) in player_locations.iter().enumerate() {
        // these values are guesses btw
        const MIN_TRACK_X: f32 = -113.0;
        const MAX_TRACK_X: f32 = 37.0;
        const MIN_TRACK_Z: f32 = -39.0;
        const MAX_TRACK_Z: f32 = 111.0;

        // translate a floating point world location into an integer pixel location
        let map_x = map_width as f32 * (location.0 - MIN_TRACK_X) / (MAX_TRACK_X - MIN_TRACK_X);
        let map_z = map_height as f32 * (location.1 - MIN_TRACK_Z) / (MAX_TRACK_Z - MIN_TRACK_Z);

        // No clue why this is needed, but it works ¯\_(ツ)_/¯
        let map_location_x = map_width as i32 - map_z as i32;
        let map_location_z = map_x as i32;

        let player_dot_color: Rgb<u8> = match player_index {
            0 => Rgb::from([230, 50, 30]),  // reddish
            1 => Rgb::from([230, 210, 10]), // yellowish
            2 => Rgb::from([20, 160, 50]),  // greenish
            3 => Rgb::from([50, 130, 220]), // blueish
            _ => Rgb::from([69, 69, 69]),   // nice
        };

        // draw a lil square around the location
        for pixel_x in (map_location_x - 5)..=(map_location_x + 5) {
            for pixel_z in (map_location_z - 5)..=(map_location_z + 5) {
                if pixel_x >= 0
                    && pixel_x as u32 <= map_width
                    && pixel_z >= 0
                    && pixel_z as u32 <= map_height
                {
                    base_map.put_pixel(
                        pixel_x.try_into().unwrap(),
                        pixel_z.try_into().unwrap(),
                        player_dot_color,
                    );
                }
            }
        }
    }

    return base_map;
}
