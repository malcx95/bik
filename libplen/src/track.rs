use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
pub struct MapConfig {
    start_position: (usize, usize),
}

