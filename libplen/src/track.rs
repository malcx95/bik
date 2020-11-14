use serde_derive::{Deserialize, Serialize};

use crate::math::Vec2;
use crate::powerup::Powerup;

#[derive(Serialize, Deserialize, Clone)]
pub struct MapConfig {
    pub start_position: Vec2,
    pub powerups: Vec<Powerup>,
    pub checkpoints: Vec<Vec2>,
}
