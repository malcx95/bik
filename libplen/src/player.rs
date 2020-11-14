use serde_derive::{Serialize, Deserialize};

use crate::constants::{
    WHEEL_DISTANCE,
    STEERING_MAX,
    ACCELERATION,
    MAX_SPEED,
    MAX_BACKWARD_SPEED,
    STEERING_RATE,
    BIKE_SCALE,
    STEERING_ATTENUATION_MAX
};
use crate::math::{Vec2, vec2};
use crate::messages::ClientInput;
use crate::constants;
use crate::powerup::Powerup;
use crate::ground::Ground;


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
    pub health: i16,

    pub time_to_next_collision: f32
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
            speed: 0.,
            steering_angle: 0.,
            lap: 0,
            checkpoint: 0,
            fuel_level: constants::INITIAL_FUEL_LEVEL,
            health: 100,
            time_to_next_collision: constants::COLLISION_GRACE_PERIOD
        }
    }

    pub fn update_fuel_level(&mut self, input: &ClientInput) {
        self.fuel_level = (self.fuel_level - input.y_input.max(0.)*constants::FUEL_CONSUMPTION).max(0.);
    }

    pub fn update(&mut self, input: &ClientInput, ground: &Ground, delta_time: f32) {
        let ground_type = ground.query_terrain(self.position)
            .expect(&format!("failed to query terrain for player {:?}", self.name));

        let motor_acceleration = input.y_input * ACCELERATION;
        let braking_force = self.speed * (1. - ground_type.braking_factor());
        let acceleration = motor_acceleration - braking_force;
        println!("{} {} {} {}", motor_acceleration, braking_force, acceleration, self.speed);

        // Steering logic
        self.speed = (self.speed + (acceleration * delta_time) * BIKE_SCALE)
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

    pub fn get_fuel_percentage(&self) -> f32 {
        self.fuel_level as f32 / constants::MAX_FUEL_LEVEL
    }

    pub fn update_collision_timer(&mut self, delta_time: f32) {
        self.time_to_next_collision -= delta_time;
        if self.time_to_next_collision < 0. {
            self.time_to_next_collision = 0.
        }
    }

    pub fn damage(&mut self, dmg: i16) -> bool {
        println!("took damage!");
        self.health -= dmg;

        if self.health <= 0 {
            self.health = 0;
        }

        true
    }

    pub fn has_died(&mut self) -> bool{
        self.health == 0
    }
}
