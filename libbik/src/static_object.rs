use serde_derive::{Deserialize, Serialize};

use crate::math::Vec2;

#[derive(Serialize, Deserialize, Clone)]
pub enum StaticObjectKind {
    Tree,
    Tire,
	FinishLine,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StaticObject {
    pub position: Vec2,
    pub kind: StaticObjectKind,
    pub variant: usize
}

impl StaticObject {
    pub fn above_player(&self) -> bool {
        match self.kind {
            StaticObjectKind::Tree => true,
            _ => false,
        }
    }
}
