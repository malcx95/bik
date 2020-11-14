use std::sync::mpsc::Receiver;

use serde_derive::{Serialize, Deserialize};

use crate::checkpoint::Checkpoint;
use crate::constants::{POWERUP_TIMEOUT, POWERUP_DISTANCE};
use crate::math::{Vec2, vec2, LineSegment};
use crate::player::Player;
use crate::powerup::Powerup;
use crate::track;

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    pub powerups: Vec<Powerup>,
    pub checkpoints: Vec<Checkpoint>,
    // put server side game state stuff here
}

impl GameState {
    pub fn new(powerups: Vec<Powerup>) -> GameState {
        GameState {
            players: Vec::new(),
            powerups,
            checkpoints: Vec::new(),
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

    pub fn add_checkpoint(&mut self, checkpoint: Checkpoint) {
        self.checkpoints.push(checkpoint);
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
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
