use std::sync::mpsc::Receiver;
use std::collections::HashSet;

use serde_derive::{Serialize, Deserialize};

use crate::checkpoint::Checkpoint;
use crate::constants;
use crate::math::{Vec2, vec2, LineSegment};
use crate::player::{PlayerState, Player};
use crate::powerup::Powerup;
use crate::static_object::StaticObject;
use crate::track;
use crate::weapon;

#[derive(Serialize, Deserialize, Clone)]
pub enum RaceState {
    NotStarted,
    Starting(f32),
    Started,
    Finished,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    pub powerups: Vec<Powerup>,
    pub checkpoints: Vec<Checkpoint>,
    pub start_position: Vec2,
    pub race_state: RaceState,
    pub static_objects: Vec<StaticObject>,
    pub finished_players: Vec<u64>,
}

impl GameState {
    pub fn new(
        mut powerups: Vec<Powerup>,
        start_point: Vec2,
        checkpoint_positions: &Vec<Vec2>,
        static_objects: Vec<StaticObject>
    ) -> GameState {
        for p in &mut powerups {
            p.position *= constants::MAP_SCALE;
        }

        let checkpoints = checkpoint_positions.iter().cloned().map(|pos|
            Checkpoint::new(pos * constants::MAP_SCALE)
        ).collect();

        GameState {
            players: Vec::new(),
            powerups,
            checkpoints,
            start_position: start_point,
            race_state: RaceState::NotStarted,
            static_objects,
            finished_players: Vec::new(),
        }
    }

    /**
     *  Updates the gamestate and returns
     *  (
     *  vec with player ids that got hit with bullets,
     *  vec with positions where powerups where picked up,
     *  vec with positions where lasers are fired
     *  )
     */
    pub fn update(&mut self, delta: f32) {
        self.race_state = match self.race_state {
            RaceState::Starting(time) => {
                if time - delta < 0. {
                    RaceState::Started
                } else {
                    RaceState::Starting(time - delta)
                }
            }
            RaceState::Started => {
                // update game state
                self.handle_player_collisions();
                self.handle_object_collision();
                self.handle_player_attacks();

                self.update_powerups(delta);

                let all_finished = self.update_finished_players();

                if all_finished {
                    RaceState::Finished
                } else {
                    RaceState::Started
                }
            }
            _ => self.race_state.clone()
        };
    }

    pub fn get_player_finish_position(&self, player_id: u64) -> i32 {
        let mut position = 1;
        for id in &self.finished_players {
            if *id == player_id {
                return position;
            }
            position += 1
        }
        -1
    }

    pub fn update_powerups(&mut self, delta: f32) {
        let mut i = 0;
        'powerups: loop {
            if i >= self.powerups.len() {
                break;
            }

            let powerup = &mut self.powerups[i];
            powerup.timeout = (powerup.timeout - delta).max(0.);

            if powerup.timeout <= 0. {
                for player in &mut self.players {
                    let distance = player.position.distance_to(powerup.position);
                    if distance < constants::POWERUP_DISTANCE {
                        player.take_powerup(&powerup);
                        powerup.timeout = constants::POWERUP_TIMEOUT;
                        continue 'powerups;
                    }
                }
            }

            i += 1;
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.push(player);
    }

    pub fn get_player_by_id(&self, id: u64) -> Option<&Player> {
        for player in &self.players {
            if player.id == id {
                return Some(player);
            }
        }

        None
    }

    fn handle_player_attacks(&mut self) {
        let mace_positions: Vec<Vec2> = self
            .players
            .iter()
            .filter_map(|player| {
                match &player.weapon {
                    Some(weapon::Weapon::Mace(mace)) => Some({
                        let offset = Vec2::from_direction(mace.angle, constants::MACE_RADIUS);
                        offset + player.position
                    }),
                    _ => None,
                }
            })
            .collect();

        for mace in mace_positions {
            for target in &mut self.players {
                for (c1, r1) in target.collision_points() {
                    if c1.distance_to(mace) < r1 {
                        target.crash();
                    }
                }
            }
        }
    }

    pub fn handle_player_collisions(&mut self) {
        let mut collided_players = HashSet::new();

        if !self.players.is_empty() {
            for (i, p1) in self.players[..self.players.len() - 1].iter().enumerate() {
                for p2 in &self.players[(i + 1)..] {
                    for (c1, r1) in p1.collision_points() {
                        for (c2, r2) in p2.collision_points() {
                            let distance = (c1 - c2).norm();
                            if p1.id != p2.id && distance < r1+r2 as f32 {
                                collided_players.insert(p1.id);
                                collided_players.insert(p2.id);
                            }
                        }
                    }
                }
            }
        }

        for player in &mut self.players {
            if collided_players.contains(&player.id) {
                player.crash();
            }
        }
    }

    /**
     * Adds newly finished players to finished_players vec,
     * returns whether all have finished.
     */
    fn update_finished_players(&mut self) -> bool {
        let mut all_finished = true;
        for player in &self.players {
            if !player.finished {
                all_finished = false;
            } else if !self.finished_players.contains(&player.id) {
                self.finished_players.push(player.id);
            }
        }

        all_finished
    }

    pub fn handle_object_collision(&mut self) {
        for player in &mut self.players {
            if player.state != PlayerState::Upright || player.velocity.norm() < constants::MIN_CRASH_VELOCITY {
                break;
            }
            for (c, r) in player.collision_points() {
                for object in &self.static_objects {
                    if let Some(obj_radius) = object.collision_radius() {
                        let distance = (c - object.position * constants::MAP_SCALE).norm();
                        if distance < r + obj_radius * constants::STATIC_OBJECT_SCALE {
                            player.state = crate::player::PlayerState::Falling(0, 0.)
                        }
                    }
                }
            }
        }
    }

    pub fn vector_to_checkpoint(&self, player: &Player) -> Vec2 {
        let checkpoint_pos = if player.checkpoint < self.checkpoints.len() {
            self.checkpoints[player.checkpoint].position
        }
        else {
            self.start_position
        };

        checkpoint_pos - player.position
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(Vec::new(), vec2(0., 0.), &Vec::new(), Vec::new())
    }
}
