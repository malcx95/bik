use enum_map::{enum_map, EnumMap};
use sdl2::image::LoadTexture;
use sdl2::mixer::Chunk;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

use libbik::constants;

pub struct Assets<'ttf, 'r> {
    pub font: sdl2::ttf::Font<'ttf, 'r>,
    pub race_font: sdl2::ttf::Font<'ttf, 'r>,

    pub bike_back: Texture<'r>,
    pub bike_front: Texture<'r>,
    pub driver: Texture<'r>,
    pub track: Texture<'r>,
    pub track_overlay: Texture<'r>,

    pub red_outline: Texture<'r>,
	pub finish_line: Texture<'r>,

    pub trees: Vec<Texture<'r>>,
    pub tires: Vec<Texture<'r>>,

    pub mace_pickup: Texture<'r>,

    pub menu_background: Texture<'r>,
    pub end_background: Texture<'r>,

    pub engine_sound: Chunk,
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
            track_overlay: load_tex("resources/track_overlay.png"),

            red_outline: load_tex("resources/red_outline.png"),
			finish_line: load_tex("resources/finish_line.png"),

            trees: vec![
                load_tex("resources/tree1.png"),
                load_tex("resources/tree2.png"),
                load_tex("resources/tree3.png"),
            ],
            tires: vec![load_tex("resources/tire1.png")],

            mace_pickup: load_tex("resources/mace.png"),

            engine_sound: Chunk::from_file("resources/audio/engine.ogg").unwrap(),
        };

        // Volume is on a scale from 0 to 128
        assets.engine_sound.set_volume(30);

        assets
    }
}
