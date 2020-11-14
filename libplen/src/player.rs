use std::f32::consts::PI;

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
use crate::ground::Ground;
use crate::gamestate::RaceState;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: u64,
    pub name: String,

    pub position: Vec2,
    pub angle: f32,
    pub velocity: Vec2,
    pub steering_angle: f32,

    pub lap: usize,
    pub checkpoint: usize,

    pub fuel_level: f32,
}


impl Player {
    pub fn new(
        id: u64,
        name: String,
        position: Vec2,
    ) -> Player {
        Player {
            id,
            name,
            position,
            angle: 0.,
            velocity: vec2(0., 0.),
            steering_angle: 0.,
            lap: 0,
            checkpoint: 0,
            fuel_level: constants::INITIAL_FUEL_LEVEL,
        }
    }

    pub fn update_fuel_level(&mut self, input: &ClientInput) {
        self.fuel_level = (self.fuel_level - input.y_input.max(0.)*constants::FUEL_CONSUMPTION).max(0.);
    }

    pub fn update(
        &mut self,
        input: &ClientInput,
        ground: &Ground,
        delta_time: f32,
        race_state: &RaceState
    ) {
        match race_state {
            RaceState::Started => {
                let ground_type = ground.query_terrain(self.position)
                    .expect(&format!("failed to query terrain for player {:?}", self.name));

                let acc_magnitude = (ACCELERATION * input.y_input * delta_time) * BIKE_SCALE;
                let acceleration = Vec2::from_direction(self.angle, acc_magnitude);

                // Side velocity attenuation. AKA anti-hovercraft force
                let side_decel = {
                    let side_direction = Vec2::from_direction(self.angle + PI/2., 1.);

                    let side_vel_magnitude = side_direction.dot(self.velocity);

                    let decel = ground_type.side_speed_decay() * side_vel_magnitude;

                    -side_direction * (decel * delta_time).max(side_vel_magnitude)
                };

                let uncapped_velocity = self.velocity + acceleration + side_decel;
                let vel_magnitude = uncapped_velocity.norm()
                    .max(-MAX_BACKWARD_SPEED)
                    .min(MAX_SPEED);

                self.velocity = if uncapped_velocity != vec2(0., 0.) {
                    uncapped_velocity.normalize() * vel_magnitude
                }
                else {
                    vec2(0., 0.)
                };


                self.position += self.velocity * delta_time;

                self.update_fuel_level(input);
            }
            _ => { }
        }

        let delta_angle = self.velocity.norm() * self.steering_angle.tan() / (WHEEL_DISTANCE * BIKE_SCALE);

        let steering_attenuation = (1. - self.velocity.norm() / MAX_SPEED) * (1. - STEERING_ATTENUATION_MAX)
            + STEERING_ATTENUATION_MAX;
        let steering_max = STEERING_MAX * steering_attenuation;

        let target_angle = steering_max * input.x_input;

        let steer_amount = (self.steering_angle - target_angle) * STEERING_RATE
            .max(-STEERING_RATE)
            .min(STEERING_RATE);
        self.steering_angle = (self.steering_angle - steer_amount * delta_time)
            .min(steering_max)
            .max(-steering_max);

        self.angle += delta_angle * delta_time;
    }

    pub fn take_powerup(&mut self, _powerup: &Powerup) {
        // TODO
    }

    pub fn get_fuel_percentage(&self) -> f32 {
        self.fuel_level as f32 / constants::MAX_FUEL_LEVEL
    }
}
