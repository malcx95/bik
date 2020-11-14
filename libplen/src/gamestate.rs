use std::sync::mpsc::Receiver;

use serde_derive::{Serialize, Deserialize};

use crate::checkpoint::Checkpoint;
use crate::constants::{MAP_SCALE, POWERUP_TIMEOUT, POWERUP_DISTANCE};
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
            p.position *= MAP_SCALE;
        }

        let checkpoints = checkpoint_positions.iter().cloned().map(|pos|
            Checkpoint::new(pos * MAP_SCALE)
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
    pub fn update(&mut self, delta: f32) {
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
                    if distance < POWERUP_DISTANCE {
                        player.take_powerup(&powerup);
                        powerup.timeout = POWERUP_TIMEOUT;
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
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(Vec::new(), &Vec::new())
    }
}
