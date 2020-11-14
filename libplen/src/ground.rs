use sdl2::surface::Surface;
use sdl2::pixels::PixelFormatEnum;

use crate::math::Vec2;
use crate::constants::MAP_SCALE;

#[derive(Debug, PartialEq)]
pub enum GroundError {
    UnknownKind(Vec<u8>),
    Not3Pixels,
    UnlockingFailed,
    UnknownPixelFormat(PixelFormatEnum),
}

#[derive(Debug, PartialEq)]
pub enum TerrainType {
    Road,
    Puddle,
    Sand,
    Obstacle,
}


impl TerrainType {
    /// Percentage of speed retained per second
    pub fn braking_factor(&self) -> f32 {
        match self {
            TerrainType::Road => 0.9,
            TerrainType::Puddle => 1.0,
            TerrainType::Sand => 0.1,
            TerrainType::Obstacle => 0.99,
        }
    }

    /// Rate of decay of side component of speed.
    /// A value of 1 makes any velocity not in the forward direction is removed in 1 second.
    ///
    /// A value of 0 makes the bike behave like a hovercraft
    pub fn side_speed_decay(&self) -> f32 {
        match self {
            TerrainType::Road => 10.,
            TerrainType::Puddle => 1.0,
            TerrainType::Sand => 5.,
            TerrainType::Obstacle => 1000.
        }
    }
}


pub struct Ground<'a> {
    map: Surface<'a>,
    stride: u32,
    backward_pixels: bool
}

impl<'a> Ground<'a> {
    pub fn new(map: Surface<'a>) -> Result<Self, GroundError> {
        let (stride, backward_pixels) = match map.pixel_format_enum() {
            PixelFormatEnum::ABGR8888 => Ok((4, true)),
            PixelFormatEnum::RGB24 => Ok((3, false)),
            other => Err(GroundError::UnknownPixelFormat(other))
        }?;
        Ok(Self {map, stride, backward_pixels})
    }

    pub fn query_terrain(&self, point: Vec2) -> Result<TerrainType, GroundError> {
        if point.x < 0. || point.y < 0. {
            return Ok(TerrainType::Sand)
        }
        let x = (point.x / MAP_SCALE) as u32;
        let y = (point.y / MAP_SCALE) as u32;
        let w = self.map.width();
        let h = self.map.height();

        if x >= w || y >= h {
            Ok(TerrainType::Sand)
        }
        else {
            let array_index = ((y * self.map.width() + x) * self.stride) as usize;

            let pixel = if let Some(data) = self.map.without_lock() {
                &data[array_index..(array_index+4)]
            }
            else {
                return Err(GroundError::UnlockingFailed)
            };

            let reversed = if self.backward_pixels {
                let result = pixel.iter().cloned().rev().collect::<Vec<u8>>();
                result
            }
            else {
                pixel.into()
            };

            let to_lookup = &reversed.as_slice()[0..3];
            match to_lookup {
                [255, 0,   0  ] => Ok(TerrainType::Obstacle),
                [89,  126, 206] => Ok(TerrainType::Puddle),
                [101, 81,  9  ] => Ok(TerrainType::Road),
                [255, 204, 104] => Ok(TerrainType::Sand),
                x @ [_,   _, _] => Err(GroundError::UnknownKind(x.into())),
                _ => Err(GroundError::Not3Pixels),
            }
        }
    }
}
