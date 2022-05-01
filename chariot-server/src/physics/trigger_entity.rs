use crate::physics::player_entity::PlayerEntity;
use glam::DVec3;

type BoundingBoxDimensions = [[f64; 2]; 3];

pub trait TriggerEntity {
    fn get_bounding_box(&self) -> BoundingBoxDimensions;

    fn check_bounding_box_collisions(&self, ply: &PlayerEntity) -> bool {
        let mut collision_dimensions = [false, false, false];

        for dimension in 0..=2 {
            let [min_1, max_1] = self.get_bounding_box()[dimension];
            let [min_2, max_2] = ply.bounding_box[dimension];

            if {
                (min_2 <= min_1 && min_1 <= max_2) // min_1 is inside 2
					|| (min_2 <= max_1 && max_1 <= max_2) // max_1 is inside 2
					|| (min_1 <= min_2 && min_2 <= max_1) // min_2 is inside 1
					|| (min_1 <= max_2 && max_2 <= max_1) // max_2 is inside 1
            } {
                collision_dimensions[dimension] = true;
            }
        }

        return collision_dimensions.iter().all(|&x| x);
    }

    fn trigger(&self, ply: &mut PlayerEntity);
}
