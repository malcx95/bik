use sdl2::render::Canvas;
use sdl2::video::Window;

use libplen::constants;
use libplen::gamestate::GameState;
use libplen::math::{self, vec2, Vec2};

use crate::assets::Assets;
use crate::rendering;

pub struct ClientState {
    // add client side state
}

impl ClientState {
    pub fn new() -> ClientState {
        ClientState {
            // init client stuff
        }
    }

    pub fn update(&mut self, _delta_time: f32, _game_state: &GameState, _my_id: u64) {
        // update client side stuff
    }

    pub fn draw(
        &self,
        _my_id: u64,
        game_state: &GameState,
        canvas: &mut Canvas<Window>,
        assets: &mut Assets,
    ) -> Result<(), String> {
        let (_screen_w, _screen_h) = canvas.logical_size();
        // let screen_center = vec2(screen_w as f32 * 0.5, screen_h as f32 * 0.5);

        // draw some stuff
        for player in &game_state.players {
            rendering::draw_texture_rotated(
                canvas,
                &assets.motorcycle,
                player.position,
                player.angle,
            )
            .unwrap();
        }

        Ok(())
    }
}
