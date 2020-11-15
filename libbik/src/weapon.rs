use serde_derive::{Serialize, Deserialize};
use crate::powerup;
use crate::constants;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mace {
    pub angle: f32,
    durability: f32,
}

impl Default for Mace {
    fn default() -> Self {
        Self {
            angle: 0.0,
            durability: 10.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Weapon {
    Mace(Mace),
}

impl Weapon {
    pub fn update(&mut self, delta_time: f32) {
        match self {
            Self::Mace(mace) => {
                mace.angle += constants::MACE_SPEED * delta_time;
                mace.durability = (mace.durability - delta_time).max(0.);
            }
        }
    }

    pub fn expired(&self) -> bool {
        match self {
            Self::Mace(mace) => mace.durability <= 0.,
        }
    }
}

impl From<&powerup::Weapon> for Weapon {
    fn from(powerup: &powerup::Weapon) -> Self {
        match powerup {
            powerup::Weapon::Mace => Self::Mace(Mace::default()),
        }
    }
}
