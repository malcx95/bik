use enum_map::{enum_map, EnumMap};
use sdl2::image::LoadTexture;
use sdl2::mixer::Chunk;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

use libplen::constants;

pub struct Assets<'ttf, 'r> {
    pub font: sdl2::ttf::Font<'ttf, 'r>,
    pub race_font: sdl2::ttf::Font<'ttf, 'r>,

    pub bike_back: Texture<'r>,
    pub bike_front: Texture<'r>,
    pub driver: Texture<'r>,
    pub track: Texture<'r>,

    pub mace_pickup: Texture<'r>,

    pub menu_background: Texture<'r>,
    pub end_background: Texture<'r>,

    pub achtung_blitzkrieg_engine: Chunk,
    pub el_pollo_romero_engine: Chunk,
    pub howdy_cowboy_engine: Chunk,
    pub suka_blyat_engine: Chunk,
    pub explosion: Chunk,
    pub powerup: Chunk,
    pub gun: Chunk,
    pub laser_fire_sound: Chunk,
    pub laser_charge_sound: Chunk,
}

impl<'ttf, 'r> Assets<'ttf, 'r> {
    pub fn new(
        texture_creator: &'r TextureCreator<WindowContext>,
        ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext,
    ) -> Assets<'ttf, 'r> {
        let load_tex = |path: &str| {
            let mut tex = texture_creator
                .load_texture(path)
                .expect(&format!("Could not load {}", path));
            tex.set_blend_mode(sdl2::render::BlendMode::Blend);
            tex
        };

        let mut assets = Assets {
            font: ttf_context
                .load_font("resources/yoster.ttf", 15)
                .expect("Could not find font!"),
            race_font: ttf_context
                .load_font("resources/RacingSansOne-Regular.ttf", 30)
                .expect("Could not find font!"),
            menu_background: load_tex("resources/menu_background.png"),
            end_background: load_tex("resources/endscreen.png"),

            bike_back: load_tex("resources/back.png"),
            bike_front: load_tex("resources/front.png"),
            driver: load_tex("resources/driver.png"),
            track: load_tex("resources/track.png"),

            mace_pickup: load_tex("resources/mace.png"),

            achtung_blitzkrieg_engine: Chunk::from_file(
                "resources/audio/achtungblitzkrieg-engine.ogg",
            )
            .unwrap(),
            el_pollo_romero_engine: Chunk::from_file("resources/audio/elpolloromero-engine.ogg")
                .unwrap(),
            howdy_cowboy_engine: Chunk::from_file("resources/audio/howdycowboy-engine.ogg")
                .unwrap(),
            suka_blyat_engine: Chunk::from_file("resources/audio/sukablyat-engine.ogg").unwrap(),
            powerup: Chunk::from_file("resources/audio/powerup.ogg").unwrap(),
            explosion: Chunk::from_file("resources/audio/explosion.ogg").unwrap(),
            gun: Chunk::from_file("resources/audio/gun.ogg").unwrap(),
            laser_fire_sound: Chunk::from_file("resources/audio/laserfire.ogg").unwrap(),
            laser_charge_sound: Chunk::from_file("resources/audio/lasercharge.ogg").unwrap(),
        };

        // Volume is on a scale from 0 to 128
        assets.achtung_blitzkrieg_engine.set_volume(30);
        assets.el_pollo_romero_engine.set_volume(30);
        assets.howdy_cowboy_engine.set_volume(30);
        assets.suka_blyat_engine.set_volume(30);

        assets
    }
}
