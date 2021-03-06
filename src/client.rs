#![allow(unused_imports)]
mod assets;
mod client_state;
mod menu;
mod pitch_effect;
mod rendering;

use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Instant;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::render::BlendMode;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::video::Window;
use structopt::StructOpt;

use assets::Assets;
use libbik::constants;
use libbik::gamestate;
use libbik::gamestate::RaceState;
use libbik::math::{vec2, Vec2};
use libbik::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage, SoundEffect};
use menu::MenuState;
use pitch_effect::{start_pitch_effect, PitchEffect};

#[derive(StructOpt)]
struct Opt {
    /// Address of server to connect to
    #[structopt(short, long, default_value = "localhost")]
    address: String,
    #[structopt(short, long, default_value = "4444")]
    port: u16,
}

fn send_client_message(msg: &ClientMessage, stream: &mut TcpStream) {
    let data = bincode::serialize(msg).expect("Failed to encode message");
    let length = data.len() as u16;
    stream
        .write_all(&length.to_be_bytes())
        .expect("Failed to send message length to server");
    stream
        .write_all(&data)
        .expect("Failed to send message to server");
}

#[derive(PartialEq)]
enum StateResult {
    Continue,
    GotoNext,
}

struct MainState {
    my_id: u64,
    game_state: gamestate::GameState,
    client_state: client_state::ClientState,
    last_time: Instant,
}

impl MainState {
    fn new(my_id: u64) -> MainState {
        MainState {
            my_id,
            game_state: gamestate::GameState::default(),
            client_state: client_state::ClientState::new(my_id),
            last_time: Instant::now(),
        }
    }

    fn update(
        &mut self,
        server_reader: &mut MessageReader,
        keyboard_state: &sdl2::keyboard::KeyboardState,
        assets: &mut Assets,
    ) -> StateResult {
        let elapsed = self.last_time.elapsed();
        self.last_time = Instant::now();
        let dt_duration = std::time::Duration::from_millis(1000 / 60);
        if elapsed < dt_duration {
            std::thread::sleep(dt_duration - elapsed);
        }

        server_reader.fetch_bytes().unwrap();

        for message in server_reader.iter() {
            match bincode::deserialize(&message).unwrap() {
                ServerMessage::AssignId(_) => panic!("Got new ID after intialisation"),
                ServerMessage::GameState(state) => self.game_state = state,
                ServerMessage::PlaySound(sound, _pos) => play_sound(match sound {
                    SoundEffect::StartRace => &assets.race_start_sound,
                    SoundEffect::Nitro => &assets.nitro_sound,
                    SoundEffect::Crash => &assets.hit_sound,
                }),
            }
        }

        let mut input = ClientInput::new();
        if keyboard_state.is_scancode_pressed(Scancode::W) {
            input.y_input += 1.0;
        }
        if keyboard_state.is_scancode_pressed(Scancode::S) {
            input.y_input -= 1.0;
        }

        if keyboard_state.is_scancode_pressed(Scancode::A) {
            input.x_input -= 1.0;
        }
        if keyboard_state.is_scancode_pressed(Scancode::D) {
            input.x_input += 1.0;
        }

        if keyboard_state.is_scancode_pressed(Scancode::E) {
            input.activate_powerup = true;
        } else {
            input.activate_powerup = false;
        }

        self.client_state
            .update(elapsed.as_secs_f32(), &self.game_state, self.my_id);

        let input_message = ClientMessage::Input(input);
        send_client_message(&input_message, &mut server_reader.stream);

        StateResult::Continue
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>, assets: &mut Assets) -> Result<(), String> {
        self.client_state
            .draw(self.my_id, &self.game_state, canvas, assets)
    }

    fn draw_ui(&mut self, canvas: &mut Canvas<Window>, assets: &mut Assets) -> Result<(), String> {
        self.client_state
            .draw_ui(self.my_id, &self.game_state, canvas, assets)?;
        Ok(())
    }

    fn get_first_game_state(&mut self, server_reader: &mut MessageReader) {
        let mut attempts = 0;

        loop {
            server_reader.fetch_bytes().unwrap();
            for message in server_reader.iter() {
                match bincode::deserialize(&message).unwrap() {
                    ServerMessage::GameState(state) => {
                        self.game_state = state;
                        match self.game_state.get_player_by_id(self.my_id) {
                            // got a game state with out ID in it
                            Some(_) => {
                                return;
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        println!("Nope");
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
            attempts += 1;
            if attempts > 20 {
                panic!("Never recieved game state from server!");
            }
        }
    }
}

fn play_sound(chunk: &sdl2::mixer::Chunk) {
    if let Err(e) = sdl2::mixer::Channel::all().play(chunk, 0) {
        println!("SDL mixer error: {}", e);
    }
}

pub fn main() -> Result<(), String> {
    let opts = Opt::from_args();
    let host = format!("{}:{}", opts.address, opts.port);
    let stream = TcpStream::connect(host).expect("Could not connect to server");
    println!("Connected to server");

    stream
        .set_nonblocking(true)
        .expect("Could not set socket as nonblocking");
    let mut reader = MessageReader::new(stream);

    let msg = loop {
        reader.fetch_bytes().unwrap();
        if let Some(msg) = reader.iter().next() {
            break bincode::deserialize(&msg).unwrap();
        }
    };

    let my_id = if let ServerMessage::AssignId(id) = msg {
        println!("Received the id {}", id);
        id
    } else {
        panic!("Expected to get an id from server")
    };

    let sdl = sdl2::init().expect("Could not initialize SDL");
    let video_subsystem = sdl.video().expect("Could not initialize SDL video");

    let window = video_subsystem
        .window(
            "very nice gem",
            constants::WINDOW_SIZE as u32,
            constants::WINDOW_SIZE as u32,
        )
        .resizable()
        .build()
        .expect("Could not create window");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("Could not create canvas");
    canvas.set_blend_mode(BlendMode::Blend);
    let texture_creator = canvas.texture_creator();

    let _audio = sdl.audio().expect("Could not initialize SDL audio");
    let frequency = 44_100;
    let format = sdl2::mixer::AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = sdl2::mixer::DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1_024;
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size)
        .expect("Could not open SDL mixer audio");
    let _mixer_context =
        sdl2::mixer::init(sdl2::mixer::InitFlag::OGG).expect("Could not initialize SDL mixer");

    // Allows 64 sounds to play simultaneously
    sdl2::mixer::allocate_channels(64);

    let ttf_context = sdl2::ttf::init().expect("Could not initialize SDL ttf");

    let mut assets = Assets::new(&texture_creator, &ttf_context);

    let mut name = whoami::username();

    let mut event_pump = sdl.event_pump().expect("Could not get event pump");

    'mainloop: loop {
        let menu_state = &mut MenuState::new();

        video_subsystem.text_input().start();
        menu_state.name = name;

        'menuloop: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'mainloop,
                    Event::KeyDown {
                        keycode: Some(kc), ..
                    } => match kc {
                        Keycode::Return => {
                            break 'menuloop;
                        }
                        Keycode::Backspace => {
                            menu_state.name.pop();
                        }
                        _ => {}
                    },
                    Event::TextInput { text, .. } => {
                        if menu_state.name.chars().count() < 20 {
                            menu_state.name += &text;
                        }
                    }
                    _ => {}
                }
            }
            rendering::setup_coordinates(&mut canvas)?;

            // Ignore all messages so we don't freeze the server
            reader.fetch_bytes().unwrap();
            for _ in reader.iter() {}

            menu_state.update();

            menu_state.draw(&mut canvas, &assets).unwrap();
        }
        video_subsystem.text_input().stop();

        name = menu_state.name.clone();

        send_client_message(
            &ClientMessage::JoinGame {
                name: menu_state.name.clone(),
            },
            &mut reader.stream,
        );

        let main_state = &mut MainState::new(my_id);

        // blocks until the first game state is recieved
        main_state.get_first_game_state(&mut reader);

        let create_lowres_target = |(width, height)| {
            texture_creator
                .create_texture_target(
                    sdl2::pixels::PixelFormatEnum::RGB332,
                    width / constants::PIXEL_SCALE,
                    height / constants::PIXEL_SCALE,
                )
                .unwrap()
        };

        let mut lowres_target = create_lowres_target(canvas.output_size()?);

        let engine_channel = sdl2::mixer::Channel::all()
            .play(&assets.engine_sound, -1)
            .unwrap();

        let mut pitch_effect = PitchEffect::new(1., &assets.engine_sound);
        unsafe {
            start_pitch_effect(engine_channel, &mut pitch_effect as *mut PitchEffect);
        }

        let mut engine_on = true;

        'gameloop: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'mainloop,
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        lowres_target = create_lowres_target((width as u32, height as u32));
                    }
                    Event::KeyDown {
                        keycode: Some(kc), ..
                    } => match kc {
                        Keycode::F1 => {
                            main_state.client_state.toggle_debug_draw();
                        }
                        Keycode::Return => match main_state.game_state.race_state {
                            RaceState::NotStarted => {
                                send_client_message(&ClientMessage::StartGame, &mut reader.stream);

                                println!("Starting game!")
                            }
                            RaceState::Finished => {
                                send_client_message(
                                    &ClientMessage::RestartGame,
                                    &mut reader.stream,
                                );
                                println!("Restarting game!")
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                }
            }
            rendering::setup_coordinates(&mut canvas)?;

            canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 204, 104));

            let state_result =
                main_state.update(&mut reader, &event_pump.keyboard_state(), &mut assets);

            let player_fuel = if let Some(player) = main_state.game_state.get_player_by_id(my_id) {
                player.fuel_level
            } else {
                0.
            };
            if player_fuel <= 0. {
                engine_channel.pause();
            } else {
                engine_channel.resume();
            }

            let player_speed = if let Some(player) = main_state.game_state.get_player_by_id(my_id) {
                player.velocity.norm()
            } else {
                0.
            };
            pitch_effect.speed_factor = 1. + player_speed / constants::MAX_SPEED * 7.;

            if engine_on {
                let player_finished =
                    if let Some(player) = main_state.game_state.get_player_by_id(my_id) {
                        player.finished
                    } else {
                        true
                    };

                //if player_finished {
                //    engine_on = false;
                //}
            }

            canvas
                .with_texture_canvas(&mut lowres_target, |mut canvas| {
                    canvas.clear();
                    let scale = 1. / constants::PIXEL_SCALE as f32;
                    canvas.set_scale(scale, scale).unwrap();
                    main_state.draw(&mut canvas, &mut assets).unwrap();
                })
                .unwrap();

            canvas.copy(&lowres_target, None, None)?;

            main_state.draw_ui(&mut canvas, &mut assets).unwrap();

            canvas.present();

            if state_result == StateResult::GotoNext {
                break 'gameloop;
            }
        }
    }

    Ok(())
}
