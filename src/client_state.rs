use std::f32::consts::PI;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use libbik::constants;
use libbik::gamestate::GameState;
use libbik::gamestate::RaceState;
use libbik::math::{self, vec2, Vec2};
use libbik::player::Player;
use libbik::static_object::{StaticObject, StaticObjectKind};

use crate::assets::Assets;
use crate::rendering;
use libbik::powerup::{self, PowerupKind};
use libbik::weapon::Weapon;

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

    pub fn camera_position(&self, canvas: &Canvas<Window>, game_state: &GameState) -> Vec2 {
        let (screen_w, screen_h) = canvas.logical_size();
        let (screen_w, screen_h) = (
            screen_w * constants::PIXEL_SCALE,
            screen_h * constants::PIXEL_SCALE,
        );
        if let Some(my_player) = game_state.get_player_by_id(self.my_id) {
            my_player.position - vec2(screen_w as f32, screen_h as f32) / 2.
        } else {
            vec2(0., 0.)
        }
    }

    pub fn draw(
        &self,
        _my_id: u64,
        game_state: &GameState,
        canvas: &mut Canvas<Window>,
        assets: &mut Assets,
    ) -> Result<(), String> {
        let camera_position = self.camera_position(canvas, game_state);
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

        for object in game_state
            .static_objects
            .iter()
            .filter(|o| !o.above_player())
        {
            let asset = match object.kind {
                StaticObjectKind::Tree => &assets.trees[object.variant],
                StaticObjectKind::Tire => &assets.tires[object.variant],
                StaticObjectKind::FinishLine => &assets.finish_line,
            };
            rendering::draw_texture_rotated_and_scaled(
                canvas,
                asset,
                object.position * constants::MAP_SCALE - camera_position,
                0.,
                vec2(2., 2.),
            );
        }

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

            self.draw_weapon(player, canvas, camera_position, assets);
        }

        for powerup in &game_state.powerups {
            if powerup.timeout > 0. {
                continue;
            }

            let texture = match &powerup.kind {
                PowerupKind::Weapon(weapon) => match weapon {
                    powerup::Weapon::Mace => &assets.mace_pickup,
                },
            };

            rendering::draw_texture(canvas, texture, powerup.position - camera_position).unwrap();
        }

        rendering::draw_uncentered_scaled(
            canvas,
            &assets.track_overlay_overhead,
            -camera_position,
            vec2(constants::MAP_SCALE, constants::MAP_SCALE),
        )
        .unwrap();

        for object in game_state
            .static_objects
            .iter()
            .filter(|o| o.above_player())
        {
            let asset = match object.kind {
                StaticObjectKind::Tree => &assets.trees[object.variant],
                StaticObjectKind::Tire => &assets.tires[object.variant],
                StaticObjectKind::FinishLine => &assets.finish_line,
            };
            rendering::draw_texture_rotated_and_scaled(
                canvas,
                asset,
                object.position * constants::MAP_SCALE - camera_position,
                0.,
                vec2(2., 2.),
            );
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

            for player in &game_state.players {
                let x = player.position.x - camera_position.x as f32;
                let y = player.position.y - camera_position.y as f32;
                rendering::draw_texture_rotated(
                    canvas,
                    &assets.red_outline,
                    vec2(x, y),
                    player.angle + player.steering_angle,
                )?;
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

        // for player in &game_state.players {
        //     if player.id != my_id {
        //         rendering::draw_text(
        //             canvas,
        //             &player.name,
        //             (player.position - self.camera_position(canvas, game_state)) / constants::PIXEL_SCALE as f32,
        //             (255, 255, 255).into(),
        //             &assets.race_font
        //         ).unwrap();
        //     }
        // }

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
            &format!("{}", num as i32),
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
            "Press Enter to start race!",
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
            "Fuel level",
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

    fn draw_weapon(
        &self,
        player: &Player,
        canvas: &mut Canvas<Window>,
        camera_position: Vec2,
        assets: &mut Assets,
    ) {
        let weapon = match &player.weapon {
            Some(weapon) => weapon,
            None => return,
        };

        match weapon {
            Weapon::Mace(mace) => {
                let texture = &assets.mace_pickup;
                let offset = Vec2::from_direction(mace.angle, constants::MACE_RADIUS);
                let position = player.position + offset;
                rendering::draw_texture(canvas, texture, position - camera_position).unwrap();
            }
        }
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
