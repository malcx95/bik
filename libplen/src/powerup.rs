use serde_derive::{Serialize, Deserialize};
use crate::math::Vec2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Weapon {
    Mace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PowerupKind {
    Weapon(Weapon),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Powerup {
    pub position: Vec2,
    pub kind: PowerupKind,
}
