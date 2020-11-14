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

use libplen::constants;
use libplen::gamestate;
use libplen::math::{vec2, Vec2};
use libplen::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage, SoundEffect};
use libplen::player::Player;
use libplen::track;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    /// Kill the server if all clients disconnect
    debug_kill: bool,
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
    map_data: Surface<'a>,
    next_id: u64,
    last_time: Instant,
    opts: Opt,
    has_had_player: bool,
    map_config: track::MapConfig,
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
            map_data: Surface::from_file("resources/track.png").expect("failed to laod map data"),
            last_time: Instant::now(),
            state: gamestate::GameState::new(map_config.powerups.clone()),
            opts,
            has_had_player: false,
            map_config,
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

        self.state.update(delta_time);

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
        let sounds_to_play = vec![];

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

                        let (x, y) = self.map_config.start_position;
                        let player = Player::new(client.id, name, vec2(x as f32, y as f32));
                        self.state.add_player(player);
                    }
                    Err(_) => {
                        println!("Could not decode message from {}, deleting", client.id);
                        clients_to_delete.push(client.id);
                    }
                }
            }

            for player in &mut self.state.players {
                if player.id == client.id {
                    player.update(&client.input, delta_time);
                    break;
                }
            }

            let result = send_server_message(
                &ServerMessage::GameState(self.state.clone()),
                &mut client.message_reader.stream,
            );
            remove_player_on_disconnect!(result, client.id);
        }

        for (sound, pos) in &sounds_to_play {
            for client in self.connections.iter_mut() {
                let result = send_server_message(
                    &ServerMessage::PlaySound(*sound, *pos),
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
