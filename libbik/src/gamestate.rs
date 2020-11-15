use std::sync::mpsc::Receiver;

use serde_derive::{Serialize, Deserialize};

use crate::checkpoint::Checkpoint;
use crate::constants;
use crate::math::{Vec2, vec2, LineSegment};
use crate::player::{PlayerState, Player};
use crate::powerup::Powerup;
use crate::static_object::StaticObject;
use crate::track;


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
    pub race_state: RaceState,
    pub static_objects: Vec<StaticObject>,
    pub finished_players: Vec<u64>,
}

impl GameState {
    pub fn new(
        mut powerups: Vec<Powerup>,
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
        // update game state
        self.handle_player_collisions(delta);
        self.handle_object_collision();
        self.update_powerups(delta);
        let all_finished = self.update_finished_players();

        self.race_state = match self.race_state {
            RaceState::Starting(time) => {
                if time - delta < 0. {
                    RaceState::Started
                } else {
                    RaceState::Starting(time - delta)
                }
            }
            RaceState::Started => {
                if all_finished {
                    RaceState::Finished
                } else {
                    RaceState::Started
                }
            }
            _ => self.race_state.clone()
        };
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

    pub fn handle_player_collisions(&mut self, delta: f32) {
        let mut collided_players: Vec<(u64, String)> = vec!();

        if !self.players.is_empty() {
            for (i, p1) in self.players[..self.players.len() - 1].iter().enumerate() {
                for p2 in &self.players[(i + 1)..] {
                    for (c1, r1) in p1.collision_points() {
                        for (c2, r2) in p2.collision_points() {
                            let distance = (c1 - c2).norm();
                            if p1.id != p2.id && distance < r1+r2 as f32 {
                                collided_players.push((p1.id, p2.name.clone()));
                            }
                        }
                    }
                }
            }
        }

        let mut damaged_players = Vec::new();
        for player in &mut self.players {
            player.update_collision_timer(delta);

            for (id, attacker) in &collided_players {
                if player.id == *id && player.time_to_next_collision == 0. {
                    let took_damage = player.damage(constants::COLLISION_DAMAGE);

                    if took_damage {
                        damaged_players.push(player.id);
                    }

                    if player.is_bike_broken() {
                        // TODO: Something should happen when the bike is broken.
                        // Force a pit stop?
                    }

                    player.time_to_next_collision = constants::COLLISION_GRACE_PERIOD;
                }
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
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(Vec::new(), &Vec::new(), Vec::new())
    }
}
