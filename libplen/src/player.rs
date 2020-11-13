use serde_derive::{Serialize, Deserialize};

use crate::constants::{WHEEL_DISTANCE, STEERING_MAX};
use crate::math::{Vec2, vec2};
use crate::messages::ClientInput;


#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,

    pub position: Vec2,
    pub angle: f32,
    pub speed: f32,
    pub steering_angle: f32,
}


impl Player {
    pub fn new(
        id: u64,
        name: String
    ) -> Player {
        Player {
            id,
            name,
            position: vec2(150., 150.),
            angle: 0.,
            speed: 0.,
            steering_angle: 0.,
        }
    }

    pub fn update(&mut self, input: &ClientInput, delta_time: f32) {
        // self.angle += input.x_input * delta_time;
        self.speed += input.y_input * delta_time * 100.;
        // player update here
        self.position += Vec2::from_direction(self.angle, self.speed * delta_time);

        let delta_angle = self.speed * self.steering_angle.tan() / WHEEL_DISTANCE;

        self.steering_angle = input.x_input * STEERING_MAX;
        self.angle += delta_angle * delta_time;
    }
}
