use serde_derive::{Deserialize, Serialize};
use crate::powerup::Powerup;

#[derive(Serialize, Deserialize, Clone)]
pub struct MapConfig {
    pub start_position: (usize, usize),
    pub powerups: Vec<Powerup>,
}
