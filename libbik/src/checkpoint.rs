use serde_derive::{Serialize, Deserialize};

use crate::constants;
use crate::math::Vec2;

#[derive(Serialize, Deserialize, Clone)]
pub struct Checkpoint {
    pub position: Vec2,
}

impl Checkpoint {
    pub fn new(position: Vec2) -> Checkpoint {
        Checkpoint {
            position
        }
    }

    pub fn player_reached(&self, player_position: Vec2) -> bool {
        player_position.distance_to(self.position) < constants::CHECKPOINT_RADIUS * constants::MAP_SCALE
    }
}
