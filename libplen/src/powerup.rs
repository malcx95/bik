use serde_derive::{Serialize, Deserialize};
use crate::math::Vec2;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Weapon {
    Mace,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum PowerupKind {
    Weapon(Weapon),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Powerup {
    pub position: Vec2,
    pub kind: PowerupKind,
    pub timeout: f32,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum MaybePowerup {
    Powerup(Powerup),
    Timeout(f32),
}
