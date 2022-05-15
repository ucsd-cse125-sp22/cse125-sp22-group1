use image::{GenericImage, Rgba, RgbaImage};

use image::io::Reader as ImageReader;

fn create_minimap_image(
    base_map_path: String,
    player_locations: Vec<(f32, f32)>,
    self_player_index: usize,
) -> RgbaImage {
    let mut base_map = ImageReader::open(base_map_path)
        .expect("Could not access base map!")
        .decode()
        .expect("Could not decode base map!");

    const MINIMAP_SIDE_LENGTH: u32 = 20;

    let map_width = base_map.width();
    let map_height = base_map.height();

    let world_location_to_map_location = |world_location: (f32, f32)| -> (u32, u32) {
        // these values are total guesses btw
        const MIN_TRACK_X: f32 = -120.0;
        const MAX_TRACK_X: f32 = 30.0;
        const MIN_TRACK_Z: f32 = -120.0;
        const MAX_TRACK_Z: f32 = 30.0;

        let map_x =
            map_width as f32 * (world_location.0 - MIN_TRACK_X) / (MAX_TRACK_X - MIN_TRACK_X);
        let map_z =
            map_height as f32 * (world_location.1 - MIN_TRACK_Z) / (MAX_TRACK_Z - MIN_TRACK_Z);
        (map_x as u32, map_z as u32)
    };

    // translate a floating point world location into an integer pixel location
    let (view_center_x, view_center_z) =
        world_location_to_map_location(player_locations[self_player_index]);

    let view_min_x = view_center_x - MINIMAP_SIDE_LENGTH / 2;
    let view_min_z = view_center_z - MINIMAP_SIDE_LENGTH / 2;
    let view_max_x = view_center_x + MINIMAP_SIDE_LENGTH / 2;
    let view_max_z = view_center_z + MINIMAP_SIDE_LENGTH / 2;

    // Annoyingly, despite taking &mut self, this method counterintuitively
    // doesn't actually modify the base image (which is what we want but
    // anyways)
    let mut map_slice = base_map.crop(
        view_min_x,
        view_min_z,
        MINIMAP_SIDE_LENGTH,
        MINIMAP_SIDE_LENGTH,
    );

    // Place a dot at each player's location
    for (player_index, location) in player_locations.iter().enumerate() {
        let (map_location_x, map_location_z) = world_location_to_map_location(*location);

        // Skip any players outside the map's view
        if map_location_x < view_min_x
            || map_location_x > view_max_x
            || map_location_z < view_min_z
            || map_location_z > view_max_z
        {
            continue;
        }

        let player_dot_color = match player_index {
            0 => Rgba::from([230, 50, 30, 0]),  // reddish
            1 => Rgba::from([230, 210, 10, 0]), // yellowish
            2 => Rgba::from([20, 160, 50, 0]),  // greenish
            3 => Rgba::from([50, 130, 220, 0]), // blueish
            _ => Rgba::from([69, 69, 69, 69]),  // nice
        };

        map_slice.put_pixel(
            map_location_x - view_min_x,
            map_location_z - view_min_z,
            player_dot_color,
        );
    }

    return map_slice.to_rgba8();
}
