use serde_derive::{Serialize, Deserialize};

use crate::math::{Vec2, vec2, LineSegment};

#[derive(Serialize, Deserialize, Clone)]
pub struct Checkpoint {
    pub id: u64,
    pub line: LineSegment,
}

impl Checkpoint {
    pub fn new(
        id: u64,
        line: LineSegment,
    ) -> Checkpoint {
        Checkpoint {
            id,
            line: LineSegment::new(vec2(150., 150.), vec2(300., 300.)),
        }
    }
    
    // Make a line segment of the player's previous position and its future position, then check whether it intersects player.checkpoint + 1
    pub fn intersects(&self, other: LineSegment) -> bool {
        self.line.intersects(other)
    }
}
