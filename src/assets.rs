use enum_map::{enum_map, EnumMap};
use sdl2::image::LoadTexture;
use sdl2::mixer::Chunk;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

use libbik::constants;

pub struct Assets<'ttf, 'r> {
    pub font: sdl2::ttf::Font<'ttf, 'r>,
    pub race_font: sdl2::ttf::Font<'ttf, 'r>,
    pub mono_font: sdl2::ttf::Font<'ttf, 'r>,

    pub bike_back: Texture<'r>,
    pub bike_front: Texture<'r>,
    pub falling: Texture<'r>,
    pub more_falling: Texture<'r>,
    pub crashed: Texture<'r>,
    pub driver: Texture<'r>,
    pub track: Texture<'r>,
    pub track_overlay: Texture<'r>,
    pub track_overlay_overhead: Texture<'r>,
    pub arrow: Texture<'r>,

    pub red_outline: Texture<'r>,
    pub finish_line: Texture<'r>,

    pub trees: Vec<Texture<'r>>,
    pub tires: Vec<Texture<'r>>,
    pub barrel: Texture<'r>,

    pub mace_pickup: Texture<'r>,
    pub nitro_pickup: Texture<'r>,
    pub nitro_sound: Chunk,
    pub hit_sound: Chunk,

    pub menu_background: Texture<'r>,
    pub end_background: Texture<'r>,

    pub engine_sound: Chunk,
    pub race_start_sound: Chunk,
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
            mono_font: ttf_context
                .load_font("resources/JetBrainsMono-Regular.ttf", 30)
                .expect("Could not find font!"),
            menu_background: load_tex("resources/menu_background.png"),
            end_background: load_tex("resources/endscreen.png"),

            bike_back: load_tex("resources/back.png"),
            bike_front: load_tex("resources/front.png"),
            falling: load_tex("resources/falling.png"),
            more_falling: load_tex("resources/more_falling.png"),
            crashed: load_tex("resources/crashed.png"),

            arrow: load_tex("resources/arrow.png"),

            driver: load_tex("resources/driver.png"),
            track: load_tex("resources/track.png"),
            track_overlay: load_tex("resources/track_overlay.png"),
            track_overlay_overhead: load_tex("resources/track_overlay_overhead.png"),

            red_outline: load_tex("resources/red_outline.png"),
            finish_line: load_tex("resources/finish_line.png"),

            trees: vec![
                load_tex("resources/tree1.png"),
                load_tex("resources/tree2.png"),
                load_tex("resources/tree3.png"),
            ],
            tires: vec![load_tex("resources/tire1.png")],
            barrel: load_tex("resources/barrel.png"),

            mace_pickup: load_tex("resources/mace.png"),
            nitro_pickup: load_tex("resources/nitro.png"),
            nitro_sound: Chunk::from_file("resources/audio/nitro.ogg").unwrap(),
            hit_sound: Chunk::from_file("resources/audio/hit.ogg").unwrap(),

            engine_sound: Chunk::from_file("resources/audio/engine.ogg").unwrap(),
            race_start_sound: Chunk::from_file("resources/audio/race_start.ogg").unwrap(),
        };

        // Volume is on a scale from 0 to 128
        assets.engine_sound.set_volume(10);
        assets.race_start_sound.set_volume(100);

        assets
    }
}
