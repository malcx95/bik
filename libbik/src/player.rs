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
use crate::powerup::{self, Powerup, PowerupKind};
use crate::ground::{TerrainType, Ground};
use crate::gamestate::RaceState;
use crate::weapon::Weapon;
use std::vec::Vec;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PlayerState {
    /// Normal driving
    Upright,
    /// The player is in stage `x` of falling and has been doing so for y seconds
    Falling(usize, f32),
    /// The player is fully crashed and has been so for x seconds
    Crashed(f32),
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: u64,
    pub name: String,

    pub state: PlayerState,

    pub position: Vec2,
    pub angle: f32,
    pub velocity: Vec2,
    pub steering_angle: f32,

    pub carried_powerup: Option<PowerupKind>,
    pub nitro: f32,
    pub weapon: Option<Weapon>,

    pub lap: usize,
    pub checkpoint: usize,

    pub fuel_level: f32,

    pub time_to_next_collision: f32,

    pub total_time: f32,
    pub current_lap: f32,
    pub best_lap: f32,
    pub lap_times: Vec<f32>,

    pub finished: bool,
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
            state: PlayerState::Upright,
            velocity: vec2(0., 0.),
            steering_angle: 0.,
            nitro: 0.,
            weapon: None,
            lap: 1,
            checkpoint: 0,
            fuel_level: constants::INITIAL_FUEL_LEVEL,
            time_to_next_collision: constants::COLLISION_GRACE_PERIOD,
            total_time: 0.,
            current_lap: 1.,
            best_lap: f32::INFINITY,
            lap_times: vec!(),
            finished: false,
            carried_powerup: None,
        }
    }

    pub fn update_fuel_level(
        &mut self,
        delta_time: f32,
        throttle: f32,
        ground: &TerrainType
    ) {
        self.fuel_level = (self.fuel_level - throttle.max(0.)*constants::FUEL_CONSUMPTION*delta_time).max(0.);

        if let TerrainType::PitStop = ground {
            self.fuel_level = (self.fuel_level + constants::FUEL_PUMP_SPEED * delta_time)
                .min(constants::MAX_FUEL_LEVEL)
        }
    }

    pub fn add_lap(&mut self) {
        self.lap_times.push(self.current_lap);
        if self.current_lap < self.best_lap {
            self.best_lap = self.current_lap;
        }
        self.current_lap = 0.;
        self.lap += 1;

        self.finished = self.lap > constants::TOTAL_NUM_LAPS;

        if self.finished {
            self.velocity = vec2(0., 0.);
        }
    }

    fn update_time(&mut self, delta_time: f32) {
        if !self.finished {
            self.total_time += delta_time;
            self.current_lap += delta_time;
        }
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
                self.tick_state(delta_time);
                self.update_motion(input, ground, delta_time, race_state);

                if let Some(weapon) = &mut self.weapon {
                    weapon.update(delta_time);
                    if weapon.expired() {
                        self.weapon = None;
                    }
                }

                if input.activate_powerup {
                    self.activate_powerup();
                }

                self.update_collision_timer(delta_time);
            }
            _ => {}
        }
    }

    pub fn tick_state(&mut self, delta_time: f32) {
        // Update player state
        self.state = match self.state {
            PlayerState::Upright => PlayerState::Upright,
            PlayerState::Falling(stage, time) => {
                if time > constants::FALLING_DURATION {
                    let new_stage = stage+1;
                    if new_stage >= constants::FALLING_STAGES {
                        PlayerState::Crashed(0.)
                    }
                    else {
                        PlayerState::Falling(new_stage, time + delta_time)
                    }
                }
                else {
                    PlayerState::Falling(stage, time + delta_time)
                }
            }
            PlayerState::Crashed(time) => {
                if time > constants::CRASH_DURATION {
                    PlayerState::Upright
                }
                else {
                    PlayerState::Crashed(time + delta_time)
                }
            }
        };
    }

    fn update_motion(
        &mut self,
        input: &ClientInput,
        ground: &Ground,
        delta_time: f32,
        race_state: &RaceState,
    ) {
        let ground_type = ground.query_terrain(self.position)
            .expect(&format!("failed to query terrain for player {:?}", self.name));

        let forward_dir = Vec2::from_direction(self.angle, 1.);
        let forward_component = forward_dir.dot(self.velocity);
        let forward_decel_amount = ground_type.braking_factor() * forward_component;
        let forward_decel = -forward_dir * (forward_decel_amount * delta_time)
            .max(forward_decel_amount);

        self.nitro = (self.nitro - delta_time).max(0.);

        match race_state {
            RaceState::Started => {
                self.update_time(delta_time);

                let fuel_factor = if input.y_input.signum() == forward_component.signum() && (
                    forward_component < -MAX_WALK_SPEED ||
                    self.fuel_level <= 0. &&
                    forward_component.abs() > MAX_WALK_SPEED
                ) {
                    0.0
                } else {
                    1.0
                };

                let (throttle, steer_command) = match self.state {
                    PlayerState::Upright => (input.y_input, input.x_input),
                    PlayerState::Crashed(_) | PlayerState::Falling(_, _) => (0., 0.)
                };

                let acc_magnitude = ACCELERATION *
                    BIKE_SCALE *
                    throttle *
                    fuel_factor *
                    if self.nitro > 0. && self.fuel_level > 0. {
                        constants::NITRO_SPEED_FACTOR
                    } else {
                        1.
                    } *
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

                self.update_fuel_level(delta_time, throttle, &ground_type);

                // Handle steering
                let delta_angle = fwd_vel_magnitude * self.steering_angle.tan() / (WHEEL_DISTANCE * BIKE_SCALE);

                let steering_attenuation = (1. - forward_component / MAX_SPEED) * (1. - STEERING_ATTENUATION_MAX)
                    + STEERING_ATTENUATION_MAX;
                let steering_max = STEERING_MAX * steering_attenuation;

                let target_angle = steering_max * steer_command;

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

    pub fn take_powerup(&mut self, powerup: &Powerup) {
        self.carried_powerup = Some(powerup.kind.clone());
    }

    pub fn activate_powerup(&mut self) {
        match &self.carried_powerup {
            Some(PowerupKind::Weapon(weapon)) => {
                self.weapon = Some(weapon.into());
            }
            Some(PowerupKind::Nitro(amount)) => {
                self.nitro += amount;
            }
            None => {}
        }
        self.carried_powerup = None;
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

    pub fn crash(&mut self) -> bool {
        if self.time_to_next_collision > 0. {
            return false;
        }


        self.time_to_next_collision = constants::COLLISION_GRACE_PERIOD;
        self.state = PlayerState::Falling(0, 0.);

        true
    }

    /// Returns a list of points where collisions should be checked. Anything inside
    /// the specified radius counts as a collision
    pub fn collision_points(&self) -> Vec<(Vec2, f32)> {
        let direction = Vec2::from_direction(self.angle, 1.);
        [
            (direction * 20., 7.),
            (vec2(0., 0.), 10.),
            (-direction * 20., 7.),
        ]
            .iter()
            .map(|(point, distance)| (self.position + point.clone() * 2., distance * 2.))
            .collect::<Vec<_>>()
    }
}
