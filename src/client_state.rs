use std::f32::consts::PI;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use libplen::constants;
use libplen::gamestate::GameState;
use libplen::gamestate::RaceState;
use libplen::math::{self, vec2, Vec2};
use libplen::player::Player;

use crate::assets::Assets;
use crate::rendering;
use libplen::powerup::{PowerupKind, Weapon};

pub struct ClientState {
    my_id: u64,
    debug_drawing: bool,
    clock: f32,
}

impl ClientState {
    pub fn new(my_id: u64) -> ClientState {
        ClientState {
            my_id,
            debug_drawing: false,
            clock: 0.,
        }
    }

    pub fn toggle_debug_draw(&mut self) {
        self.debug_drawing = !self.debug_drawing;
    }

    pub fn update(&mut self, delta_time: f32, _game_state: &GameState, _my_id: u64) {
        self.clock += delta_time;
    }

    pub fn draw(
        &self,
        _my_id: u64,
        game_state: &GameState,
        canvas: &mut Canvas<Window>,
        assets: &mut Assets,
    ) -> Result<(), String> {
        let (screen_w, screen_h) = canvas.logical_size();
        let (screen_w, screen_h) = (
            screen_w * constants::PIXEL_SCALE,
            screen_h * constants::PIXEL_SCALE,
        );
        let camera_position = if let Some(my_player) = game_state.get_player_by_id(self.my_id) {
            my_player.position - vec2(screen_w as f32, screen_h as f32) / 2.
        } else {
            vec2(0., 0.)
        };

        // let screen_center = vec2(screen_w as f32 * 0.5, screen_h as f32 * 0.5);

        rendering::draw_uncentered_scaled(
            canvas,
            &assets.track,
            -camera_position,
            vec2(constants::MAP_SCALE, constants::MAP_SCALE),
        )
        .unwrap();
        rendering::draw_uncentered_scaled(
            canvas,
            &assets.track_overlay,
            -camera_position,
            vec2(constants::MAP_SCALE, constants::MAP_SCALE),
        )
        .unwrap();

        // draw some stuff
        for player in &game_state.players {
            rendering::draw_texture_rotated_and_scaled(
                canvas,
                &assets.bike_back,
                player.position - camera_position,
                player.angle + PI / 2.,
                vec2(constants::BIKE_SCALE, constants::BIKE_SCALE),
            )
            .unwrap();

            let front_offset = Vec2::from_direction(player.angle, constants::WHEEL_DISTANCE)
                * constants::BIKE_SCALE;

            rendering::draw_texture_rotated_and_scaled(
                canvas,
                &assets.bike_front,
                player.position + front_offset - camera_position,
                player.angle + PI / 2. + player.steering_angle,
                vec2(constants::BIKE_SCALE, constants::BIKE_SCALE),
            )
            .unwrap();

            rendering::draw_texture_rotated_and_scaled(
                canvas,
                &assets.driver,
                player.position - camera_position,
                player.angle + PI / 2.,
                vec2(constants::BIKE_SCALE, constants::BIKE_SCALE),
            )
            .unwrap();
        }

        for powerup in &game_state.powerups {
            if powerup.timeout > 0. {
                continue;
            }

            let texture = match &powerup.kind {
                PowerupKind::Weapon(weapon) => match weapon {
                    Weapon::Mace => &assets.mace_pickup,
                },
            };

            rendering::draw_texture(canvas, texture, powerup.position - camera_position).unwrap();
        }

        if self.debug_drawing {
            canvas.set_draw_color((255, 0, 0));
            for checkpoint in &game_state.checkpoints {
                let x = checkpoint.position.x - camera_position.x - constants::CHECKPOINT_RADIUS;
                let y = checkpoint.position.y - camera_position.y - constants::CHECKPOINT_RADIUS;
                canvas
                    .draw_rect(Rect::new(
                        x as i32,
                        y as i32,
                        (constants::CHECKPOINT_RADIUS * 2.) as u32,
                        (constants::CHECKPOINT_RADIUS * 2.) as u32,
                    ))
                    .unwrap();
            }
        }

        Ok(())
    }

    fn draw_lap_info(
        canvas: &mut Canvas<Window>,
        assets: &Assets,
        lap: usize,
    ) -> Result<(), String> {
        let text = assets
            .race_font
            .render(&format!("Lap: {}", lap))
            .blended((255, 255, 255))
            .expect("Could not render text");

        let texture_creator = canvas.texture_creator();
        let text_texture = texture_creator.create_texture_from_surface(text).unwrap();

        let res_offset = rendering::calculate_resolution_offset(canvas);
        rendering::draw_texture(
            canvas,
            &text_texture,
            vec2(constants::LAP_POS.0, constants::LAP_POS.1) + res_offset,
        )
        .unwrap();
        Ok(())
    }

    pub fn draw_ui(
        &self,
        my_id: u64,
        game_state: &GameState,
        canvas: &mut Canvas<Window>,
        assets: &mut Assets,
    ) -> Result<(), String> {
        if let Some(player) = game_state.get_player_by_id(my_id) {
            Self::draw_lap_info(canvas, assets, player.lap).unwrap();
        }

        let (screen_w, screen_h) = canvas.logical_size();
        let screen_center = vec2(screen_w as f32 * 0.5, screen_h as f32 * 0.5);

        let player = game_state.get_player_by_id(my_id).unwrap();

        self.draw_fuel_gauge(player, canvas, screen_center, assets);

        match game_state.race_state {
            RaceState::NotStarted => {
                self.draw_pre_race_text(canvas, assets);
            }
            RaceState::Starting(t) => {
                self.draw_race_countdown(canvas, assets, t);
            }
            _ => {}
        }

        Ok(())
    }

    fn draw_race_countdown(
        &self,
        canvas: &mut Canvas<Window>,
        assets: &mut Assets,
        countdown_time: f32,
    ) {
        let num = countdown_time.ceil();
        let round_err = 1. - (num - countdown_time);

        let size = constants::COUNTDOWN_TEXT_MIN_SIZE
            + (constants::COUNTDOWN_TEXT_MAX_SIZE - constants::COUNTDOWN_TEXT_MIN_SIZE) * round_err;

        let (screen_w, screen_h) = canvas.logical_size();
        let pos = vec2(
            screen_w as f32 * 0.5,
            screen_h as f32 * constants::PRE_RACE_PRESS_ENTER_POS_Y,
        );

        rendering::draw_text_rotated_and_scaled(
            canvas,
            format!("{}", num as i32),
            pos,
            (255, 255, 255).into(),
            &assets.race_font,
            0.,
            vec2(size, size),
        )
        .unwrap();
    }

    fn draw_pre_race_text(&self, canvas: &mut Canvas<Window>, assets: &mut Assets) {
        let (screen_w, screen_h) = canvas.logical_size();
        let pos = vec2(
            screen_w as f32 * 0.5,
            screen_h as f32 * constants::PRE_RACE_PRESS_ENTER_POS_Y,
        );
        let oscillation_size = 0.8 + ((self.clock.sin() + 1.) / 2.) * 0.2;

        rendering::draw_text_rotated_and_scaled(
            canvas,
            String::from("Press Enter to start race!"),
            pos,
            (255, 255, 255).into(),
            &assets.race_font,
            (self.clock / 2.).sin() / 16.,
            vec2(oscillation_size, oscillation_size),
        )
        .unwrap();
    }

    fn draw_fuel_gauge(
        &self,
        player: &Player,
        canvas: &mut Canvas<Window>,
        _screen_center: Vec2,
        assets: &mut Assets,
    ) {
        let (screen_w, screen_h) = canvas.logical_size();

        let gauge_pos_x = (constants::GAUGE_POS_X * (screen_w as f32)) as i32;
        let gauge_pos_y = (constants::GAUGE_POS_Y * (screen_h as f32)) as i32;

        let padding = constants::GAUGE_TEXT_POS_PADDING * (screen_h as f32);

        rendering::draw_text(
            canvas,
            String::from("Fuel level"),
            vec2(gauge_pos_x as f32 + 30., gauge_pos_y as f32 - padding),
            (255, 255, 255).into(),
            &assets.font,
        )
        .unwrap();

        let fuel_bar_height =
            (constants::GAUGE_HEIGHT * (screen_h as f32) * player.get_fuel_percentage()) as i32;
        let max_fuel_bar_height = (constants::GAUGE_HEIGHT * (screen_h as f32)) as i32;

        canvas.set_draw_color(self.get_fuel_bar_color(player));
        canvas
            .fill_rect(Rect::new(
                gauge_pos_x,
                gauge_pos_y + (max_fuel_bar_height - fuel_bar_height),
                (constants::GAUGE_WIDTH * (screen_w as f32)) as u32,
                fuel_bar_height as u32,
            ))
            .unwrap();

        canvas.set_draw_color(constants::GAUGE_BACKGROUND);
        canvas
            .draw_rect(Rect::new(
                gauge_pos_x,
                gauge_pos_y,
                (constants::GAUGE_WIDTH * (screen_w as f32)) as u32,
                max_fuel_bar_height as u32,
            ))
            .unwrap();
    }

    fn get_fuel_bar_color(&self, player: &Player) -> (u8, u8, u8) {
        let fuel_percentage = player.get_fuel_percentage();
        let r: f32;
        let g: f32;
        let b: f32;
        if fuel_percentage > 0.5 {
            r = 255. * (1. - (fuel_percentage - 0.5) * 2.);
            g = 255.;
        } else {
            r = 255.;
            g = 255. * fuel_percentage * 2.;
        }
        b = 0.;
        (r as u8, g as u8, b as u8)
    }
}
