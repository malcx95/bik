use std::sync::mpsc::Receiver;

use serde_derive::{Serialize, Deserialize};

use crate::checkpoint::Checkpoint;
use crate::constants;
use crate::math::{Vec2, vec2, LineSegment};
use crate::player::Player;
use crate::powerup::Powerup;
use crate::track;


#[derive(Serialize, Deserialize, Clone)]
pub enum RaceState {
    NotStarted,
    Starting(f32),
    Started,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    pub powerups: Vec<Powerup>,
    pub checkpoints: Vec<Checkpoint>,
    pub race_state: RaceState,
}

impl GameState {
    pub fn new(mut powerups: Vec<Powerup>, checkpoint_positions: &Vec<Vec2>) -> GameState {
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
    pub fn update(&mut self, delta: f32) -> Vec<u64> {
        // update game state
        let hit_players = self.handle_player_collisions(delta);
        self.update_powerups(delta);

        self.race_state = match self.race_state {
            RaceState::Starting(time) => {
                if time - delta < 0. {
                    RaceState::Started
                } else {
                    RaceState::Starting(time - delta)
                }
            }
            _ => self.race_state.clone()
        };

        hit_players
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

    pub fn get_checkpoint_by_id(&self, id: u64) -> Option<&Checkpoint> {
        for checkpoint in &self.checkpoints {
            if checkpoint.id == id {
                return Some(checkpoint);
            }
        }

        None
    }

    pub fn handle_player_collisions(&mut self, delta: f32) -> Vec<u64> {
        let mut collided_players: Vec<(u64, String)> = vec!();
        let hit_radius = constants::BIKE_SIZE * 2;

        for p1 in &self.players {
            for p2 in &self.players {
                let distance = (p1.position - p2.position).norm();
                if p1.id != p2.id && distance < hit_radius as f32 {
                    collided_players.push((p1.id, p2.name.clone()));
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

					player.speed -= constants::COLLISION_SPEED_REDUCTION;
                    player.time_to_next_collision = constants::COLLISION_GRACE_PERIOD;
                }
            }
        }
        damaged_players
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(Vec::new(), &Vec::new())
    }
}
