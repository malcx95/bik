use serde_derive::{Serialize, Deserialize};

use crate::constants::{
    WHEEL_DISTANCE,
    STEERING_MAX,
    ACCELERATION,
    MAX_SPEED,
    MAX_BACKWARD_SPEED,
    STEERING_RATE,
    BIKE_SCALE,
    STEERING_ATTENUATION_MAX,
};
use crate::math::{Vec2, vec2};
use crate::messages::ClientInput;
use crate::constants;
use crate::powerup::Powerup;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: u64,
    pub name: String,

    pub position: Vec2,
    pub angle: f32,
    pub speed: f32,
    pub steering_angle: f32,

    pub lap: u64,
    pub checkpoint: u64,

    pub fuel_level: f32,
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
            lap: 0,
            checkpoint: 0,
            fuel_level: constants::INITIAL_FUEL_LEVEL,
        }
    }

    pub fn update_fuel_level(&mut self, input: &ClientInput) {
        self.fuel_level = (self.fuel_level - input.y_input.max(0.)*constants::FUEL_CONSUMPTION).max(0.);
    }

    pub fn update(&mut self, input: &ClientInput, delta_time: f32) {
        self.speed = (self.speed + (input.y_input * ACCELERATION * delta_time) * BIKE_SCALE)
            .max(-MAX_BACKWARD_SPEED)
            .min(MAX_SPEED);

        self.position += Vec2::from_direction(self.angle, self.speed * delta_time);

        let delta_angle = self.speed * self.steering_angle.tan() / (WHEEL_DISTANCE * BIKE_SCALE);

        let steering_attenuation = (1. - self.speed / MAX_SPEED) * (1. - STEERING_ATTENUATION_MAX)
            + STEERING_ATTENUATION_MAX;
        let steering_max = STEERING_MAX * steering_attenuation;

        // self.steering_angle = steering_max * input.x_input;

        let target_angle = steering_max * input.x_input;

        let steer_amount = (self.steering_angle - target_angle) * STEERING_RATE
            .max(-STEERING_RATE)
            .min(STEERING_RATE);
        self.steering_angle = (self.steering_angle - steer_amount * delta_time)
            .min(steering_max)
            .max(-steering_max);

        self.angle += delta_angle * delta_time;

        self.update_fuel_level(input);
    }

    pub fn take_powerup(&mut self, _powerup: &Powerup) {
        // TODO
    }
}
