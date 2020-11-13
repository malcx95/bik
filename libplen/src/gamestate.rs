use std::fs;
use std::sync::mpsc::Receiver;

use serde_derive::{Serialize, Deserialize};
use ron;

use crate::player::Player;
use crate::math::{Vec2, vec2};
use crate::track;

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
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
        // update game state
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
