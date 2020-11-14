use std::f32::consts::PI;

use serde_derive::{Serialize, Deserialize};

use crate::constants::{
    WHEEL_DISTANCE,
    STEERING_MAX,
    ACCELERATION,
    MAX_SPEED,
    MAX_WALK_SPEED,
    STEERING_RATE,
    BIKE_SCALE,
    STEERING_ATTENUATION_MAX
};
use crate::math::{Vec2, vec2};
use crate::messages::ClientInput;
use crate::constants;
use crate::powerup::Powerup;
use crate::ground::{TerrainType, Ground};
use crate::gamestate::RaceState;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: u64,
    pub name: String,

    pub position: Vec2,
    pub angle: f32,
    pub velocity: Vec2,
    pub steering_angle: f32,
    pub speed: f32,

    pub lap: usize,
    pub checkpoint: usize,

    pub fuel_level: f32,
    pub bike_health: i16,

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
            velocity: vec2(0., 0.),
            steering_angle: 0.,
            speed: 0.,
            lap: 0,
            checkpoint: 0,
            fuel_level: constants::INITIAL_FUEL_LEVEL,
            bike_health: 100,
            time_to_next_collision: constants::COLLISION_GRACE_PERIOD
        }
    }

    pub fn update_fuel_level(
        &mut self,
        delta_time: f32,
        input: &ClientInput,
        ground: &TerrainType
    ) {
        self.fuel_level = (self.fuel_level - input.y_input.max(0.)*constants::FUEL_CONSUMPTION*delta_time).max(0.);

        if let TerrainType::PitStop = ground {
            self.fuel_level = (self.fuel_level + constants::FUEL_PUMP_SPEED * delta_time)
                .min(constants::MAX_FUEL_LEVEL)
        }
    }

    pub fn update(
        &mut self,
        input: &ClientInput,
        ground: &Ground,
        delta_time: f32,
        race_state: &RaceState
    ) {
        let ground_type = ground.query_terrain(self.position)
            .expect(&format!("failed to query terrain for player {:?}", self.name));

        let forward_dir = Vec2::from_direction(self.angle, 1.);
        let forward_component = forward_dir.dot(self.velocity);
        let forward_decel_amount = ground_type.braking_factor() * forward_component;
        let forward_decel = -forward_dir * (forward_decel_amount * delta_time)
            .max(forward_decel_amount);

        match race_state {
            RaceState::Started => {
                let fuel_factor = if input.y_input.signum() == forward_component.signum() && (
                    forward_component < -MAX_WALK_SPEED ||
                    self.fuel_level <= 0. &&
                    forward_component.abs() > MAX_WALK_SPEED
                ) {
                    0.0
                } else {
                    1.0
                };

                let acc_magnitude = ACCELERATION *
                    BIKE_SCALE *
                    input.y_input *
                    fuel_factor *
                    delta_time;

                let acceleration = Vec2::from_direction(self.angle, acc_magnitude);

                // Side velocity attenuation. AKA anti-hovercraft force
                let side_direction = Vec2::from_direction(self.angle + PI/2., 1.);

                let side_decel = {
                    let side_vel_magnitude = side_direction.dot(self.velocity);

                    let decel = ground_type.side_speed_decay() * side_vel_magnitude;

                    -side_direction * (decel * delta_time).max(side_vel_magnitude)
                };

                let uncapped_velocity = self.velocity + acceleration + side_decel + forward_decel;

                let fwd_vel_magnitude = forward_dir.dot(uncapped_velocity);
                let side_vel_magnitude = side_direction.dot(uncapped_velocity);
                self.velocity = forward_dir * fwd_vel_magnitude + side_direction * side_vel_magnitude ;


                self.position += self.velocity * delta_time;

                self.update_fuel_level(delta_time, input, &ground_type);

                // Handle steering
                let delta_angle = fwd_vel_magnitude * self.steering_angle.tan() / (WHEEL_DISTANCE * BIKE_SCALE);

                let steering_attenuation = (1. - forward_component / MAX_SPEED) * (1. - STEERING_ATTENUATION_MAX)
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
            _ => { }
        }
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
        self.bike_health -= dmg;

        if self.bike_health <= 0 {
            self.bike_health = 0;
        }

        true
    }

    pub fn is_bike_broken(&mut self) -> bool{
        self.bike_health == 0
    }
}
