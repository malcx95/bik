use std::f32::consts::PI;

use sdl2::render::Canvas;
use sdl2::video::Window;

use libplen::constants;
use libplen::gamestate::GameState;
use libplen::math::{self, vec2, Vec2};

use crate::assets::Assets;
use crate::rendering;
use libplen::powerup::{PowerupKind, Weapon};

pub struct ClientState {
    // add client side state
    my_id: u64,
}

impl ClientState {
    pub fn new(my_id: u64) -> ClientState {
        ClientState { my_id }
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
        let (screen_w, screen_h) = canvas.logical_size();
        let camera_position = if let Some(my_player) = game_state.get_player_by_id(self.my_id) {
            my_player.position - vec2(screen_w as f32, screen_h as f32) / 2.
        } else {
            vec2(0., 0.)
        };

        // let screen_center = vec2(screen_w as f32 * 0.5, screen_h as f32 * 0.5);

        rendering::draw_texture(canvas, &assets.track, -camera_position).unwrap();

        // draw some stuff
        for player in &game_state.players {
            rendering::draw_texture_rotated(
                canvas,
                &assets.bike_back,
                player.position - camera_position,
                player.angle + PI / 2.,
            )
            .unwrap();

            let bike_length = 50.;
            let front_offset = Vec2::from_direction(player.angle, bike_length);

            rendering::draw_texture_rotated(
                canvas,
                &assets.bike_front,
                player.position + front_offset - camera_position,
                player.angle + PI / 2. + player.steering_angle,
            )
            .unwrap();

            rendering::draw_texture_rotated(
                canvas,
                &assets.driver,
                player.position - camera_position,
                player.angle + PI / 2.,
            )
            .unwrap();
        }

        for powerup in &game_state.powerups {
            let texture = match &powerup.kind {
                PowerupKind::Weapon(weapon) => match weapon {
                    Weapon::Mace => &assets.mace_pickup,
                },
            };

            rendering::draw_texture_rotated(canvas, texture, powerup.position, 0.).unwrap();
        }

        Ok(())
    }
}
