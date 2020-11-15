#![allow(unused_imports)]

use std::fs;
use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Instant;
use std::vec;

use sdl2::image::LoadSurface;
use sdl2::surface::Surface;
use structopt::StructOpt;
use unicode_truncate::UnicodeTruncateStr;

use libbik::constants;
use libbik::gamestate;
use libbik::gamestate::RaceState;
use libbik::ground::Ground;
use libbik::math::{vec2, LineSegment, Vec2};
use libbik::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage, SoundEffect};
use libbik::player::Player;
use libbik::track;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    /// Kill the server if all clients disconnect
    debug_kill: bool,
    /// Override the initial countdown timer
    #[structopt(short = "s", long)]
    start_countdown: Option<f32>,
}

fn send_bytes(bytes: &[u8], stream: &mut TcpStream) -> io::Result<()> {
    let mut start = 0;
    loop {
        match stream.write(&bytes[start..bytes.len()]) {
            Ok(n) => {
                if n < bytes.len() - start {
                    start += n;
                } else {
                    break Ok(());
                }
            }
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => continue,
                io::ErrorKind::Interrupted => continue,
                _ => return Err(e),
            },
        }
    }
}

fn send_server_message(msg: &ServerMessage, stream: &mut TcpStream) -> io::Result<()> {
    let data = bincode::serialize(msg).expect("Failed to encode message");
    let length = data.len() as u16;
    send_bytes(&length.to_be_bytes(), stream)?;
    send_bytes(&data, stream)
}

struct Client {
    id: u64,
    message_reader: MessageReader,
    input: ClientInput,
}

struct Server<'a> {
    listener: TcpListener,
    connections: Vec<Client>,
    state: gamestate::GameState,
    ground: Ground<'a>,
    next_id: u64,
    last_time: Instant,
    opts: Opt,
    has_had_player: bool,
    sounds_to_play: Vec<(SoundEffect, Vec2)>,
}

impl<'a> Server<'a> {
    pub fn new() -> Self {
        let opts = Opt::from_args();

        let map_config: track::MapConfig = ron::de::from_str(
            &fs::read_to_string("resources/map.ron").expect("Could not open map.ron"),
        )
        .unwrap();

        let listener = TcpListener::bind("0.0.0.0:4444").unwrap();

        listener.set_nonblocking(true).unwrap();

        println!("Listening on 0.0.0.0:4444");

        Self {
            listener,
            connections: vec![],
            next_id: 0,
            ground: Ground::new(
                Surface::from_file("resources/track.png").expect("failed to load map data"),
            )
            .expect("failed to load ground"),
            last_time: Instant::now(),
            state: gamestate::GameState::new(
                map_config.powerups.clone(),
                map_config.start_position * constants::MAP_SCALE,
                &map_config.checkpoints,
                map_config.static_objects.clone(),
            ),
            opts,
            has_had_player: false,
            sounds_to_play: vec![],
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.last_time.elapsed();
        let delta_time = constants::DELTA_TIME;
        let dt_duration = std::time::Duration::from_millis(constants::SERVER_SLEEP_DURATION);
        if elapsed < dt_duration {
            std::thread::sleep(dt_duration - elapsed);
        }
        self.last_time = Instant::now();

        let Self {
            state,
            sounds_to_play,
            ..
        } = self;
        state.update(delta_time, |sound| sounds_to_play.push(sound));

        self.accept_new_connections();
        self.update_clients(delta_time);
    }

    fn accept_new_connections(&mut self) {
        // Read data from clients
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    stream.set_nonblocking(true).unwrap();
                    println!("Got new connection {}", self.next_id);
                    let message = ServerMessage::AssignId(self.next_id);
                    if send_server_message(&message, &mut stream).is_err() {
                        println!("Could not send assign id message");
                        continue;
                    }
                    self.connections.push(Client {
                        id: self.next_id,
                        message_reader: MessageReader::new(stream),
                        input: ClientInput::new(),
                    });
                    self.has_had_player = true;
                    self.next_id += 1;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // wait until network socket is ready, typically implemented
                    // via platform-specific APIs such as epoll or IOCP
                    break;
                }
                e => {
                    e.expect("Socket listener error");
                }
            }
        }
    }

    fn update_clients(&mut self, delta_time: f32) {
        // Send data to clients
        let mut clients_to_delete = vec![];

        macro_rules! remove_player_on_disconnect {
            ($op:expr, $id:expr) => {
                match $op {
                    Ok(_) => {}
                    Err(e) => match e.kind() {
                        io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe => {
                            println!("Player {} disconnected", $id);
                            clients_to_delete.push($id);
                            break;
                        }
                        e => panic!("Unhandled network issue: {:?}", e),
                    },
                };
            };
        }

        for client in self.connections.iter_mut() {
            remove_player_on_disconnect!(client.message_reader.fetch_bytes(), client.id);

            for message in client.message_reader.iter() {
                match bincode::deserialize(&message) {
                    Ok(ClientMessage::Input(input)) => {
                        client.input = input;
                    }
                    Ok(ClientMessage::JoinGame { mut name }) => {
                        if name.trim().is_empty() {
                            name = "Mr Whitespace".into();
                        } else {
                            name = name.trim().unicode_truncate(20).0.to_string()
                        }

                        let start_distance = -50. * self.state.players.len() as f32;
                        let position = self.state.start_position + vec2(0., start_distance);

                        let player = Player::new(client.id, name, position);
                        self.state.add_player(player);
                    }
                    Ok(ClientMessage::StartGame) => {
                        let countdown = self
                            .opts
                            .start_countdown
                            .unwrap_or(constants::RACE_COUNTDOWN_TIMER_START);
                        self.state.race_state = RaceState::Starting(countdown);
                        self.sounds_to_play
                            .push((SoundEffect::StartRace, self.state.start_position));
                        println!("Client {} is starting game!", client.id);
                    }
                    Err(_) => {
                        println!("Could not decode message from {}, deleting", client.id);
                        clients_to_delete.push(client.id);
                    }
                }
            }

            for player in &mut self.state.players {
                if player.id == client.id && !player.finished {
                    let old_pos = player.position;
                    player.update(
                        &client.input,
                        &self.ground,
                        delta_time,
                        &self.state.race_state,
                    );

                    if player.checkpoint < self.state.checkpoints.len() {
                        let checkpoint = &self.state.checkpoints[player.checkpoint];
                        if checkpoint.player_reached(player.position) {
                            player.checkpoint += 1;
                        }
                    } else {
                        // let start_point = self.map_config.start_position * constants::MAP_SCALE;
                        let start_point = self.state.start_position;
                        let goal_width = constants::CHECKPOINT_RADIUS * constants::MAP_SCALE;
                        let goal_line = LineSegment::new(
                            start_point + vec2(0., -goal_width),
                            start_point + vec2(0., goal_width),
                        );

                        let player_movement_line = LineSegment::new(old_pos, player.position);
                        if player_movement_line.intersects(goal_line) {
                            player.add_lap();
                            player.checkpoint = 0;
                        }
                    }
                    break;
                }
            }

            let result = send_server_message(
                &ServerMessage::GameState(self.state.clone()),
                &mut client.message_reader.stream,
            );
            remove_player_on_disconnect!(result, client.id);
        }

        for (sound, pos) in self.sounds_to_play.drain(..) {
            for client in self.connections.iter_mut() {
                let result = send_server_message(
                    &ServerMessage::PlaySound(sound, pos),
                    &mut client.message_reader.stream,
                );
                remove_player_on_disconnect!(result, client.id);
            }
        }

        self.state
            .players
            .retain(|player| !clients_to_delete.contains(&player.id));
        self.connections
            .retain(|client| !clients_to_delete.contains(&client.id));

        if self.has_had_player && self.connections.is_empty() && self.opts.debug_kill {
            panic!("All clients disconnected and debug mode is on. Exiting")
        }
    }
}

fn main() {
    let mut server = Server::new();
    loop {
        server.update();
    }
}
