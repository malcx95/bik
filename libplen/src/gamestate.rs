use std::fs;
use std::sync::mpsc::Receiver;

use serde_derive::{Serialize, Deserialize};
use ron;

use crate::player::Player;
use crate::math::{Vec2, vec2};
use crate::track;
use crate::powerup::Powerup;
use crate::constants::POWERUP_DISTANCE;

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    pub powerups: Vec<Powerup>,
    // put server side game state stuff here
}

impl GameState {
    pub fn new() -> GameState {
        let map_config: track::MapConfig = ron::de::from_str(
            &fs::read_to_string("resources/map.ron")
                .expect("Could not open map.ron")
        ).unwrap();

        GameState {
            players: Vec::new(),
            powerups: vec![Powerup {
                position: vec2(500.0, 500.0),
                kind: crate::powerup::PowerupKind::Weapon(crate::powerup::Weapon::Mace),
            }],
            // init server side game state stuff here
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
    pub fn update(&mut self, _delta: f32) {
        for player in &mut self.players {
            let mut i = 0;
            loop {
                if i >= self.powerups.len() {
                    break;
                }

                let distance = player.position.distance_to(self.powerups[i].position);
                if distance < POWERUP_DISTANCE {
                    player.take_powerup(self.powerups.swap_remove(i));
                } else {
                    i += 1;
                }
            }
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.push(player.clone());
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
