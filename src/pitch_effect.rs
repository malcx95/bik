use libc::{c_int, c_void};
use sdl2::mixer::{Channel, Chunk};
use sdl2_sys;

// NOTE: All the code here assumes that samples can be coerced as i16
// little endian on little endian platforms

// The sample is also assumed to be looping

pub struct PitchEffect {
    pub speed_factor: f32,
    pub chunk_data: Vec<i16>,

    position: usize, // In milliseconds
}

impl PitchEffect {
    pub fn new(speed_factor: f32, chunk: &Chunk) -> Self {
        let (_, format, _) = sdl2::mixer::query_spec().unwrap();
        let sample_size = format_sample_size(format);

        let chunk_data = unsafe {
            let chunk_size = (*chunk.raw).alen as usize / sample_size;
            let chunk_slice = std::slice::from_raw_parts((*chunk.raw).abuf as *mut i16, chunk_size);
            chunk_slice.iter().cloned().collect()
        };

        Self {
            speed_factor,
            chunk_data,
            position: 0,
        }
    }
}

fn format_sample_size(format: sdl2::mixer::AudioFormat) -> usize {
    ((format & 0xFF) / 8) as usize
}

fn bytes_to_milliseconds(chunk_size: usize) -> usize {
    let (freq, format, channel_count) = sdl2::mixer::query_spec().unwrap();

    if format != sdl2::mixer::AUDIO_S16LSB {
        panic!("The code was not made to use this audio format");
    }

    let sample_points = chunk_size / format_sample_size(format);
    let sample_frames = sample_points / channel_count as usize;
    sample_frames * 1000 / freq as usize
}

unsafe extern "C" fn effect_callback(
    _chan: c_int,
    stream: *mut c_void,
    len: c_int,
    udata: *mut c_void,
) {
    let (freq, format, channel_count) = sdl2::mixer::query_spec().unwrap();
    let sample_size = format_sample_size(format);

    let buffer: &mut [i16] =
        std::slice::from_raw_parts_mut(stream as *mut i16, len as usize / sample_size);
    let effect: &mut PitchEffect = (udata as *mut PitchEffect).as_mut().unwrap();

    let duration = effect.chunk_data.len() * sample_size;

    let buffer_size = len as usize / sample_size;
    let buffer_duration = bytes_to_milliseconds(len as usize);

    let normal_delta = 1000.0 / freq as f32;
    let stretched_delta = normal_delta * effect.speed_factor;

    // j goes from 0 to buffer_size / channel_count
    // i goes from 0 to buffer_size in steps of channel_count
    for j in 0..(buffer_size / channel_count as usize) {
        let i = j * channel_count as usize;

        let x = effect.position as f32 + (j as f32) * stretched_delta;
        let k = (x / normal_delta) as usize;
        let interpolation_factor = (x / normal_delta) - k as f32;

        for c in 0..(channel_count as usize) {
            let chunk_size = effect.chunk_data.len();
            let value1 = effect.chunk_data[(k * channel_count as usize + c) % chunk_size] as f32;
            let value2 =
                effect.chunk_data[((k + 1) * channel_count as usize + c) % chunk_size] as f32;
            buffer[i + c] =
                ((1. - interpolation_factor) * value1 + interpolation_factor * value2) as i16;
        }
    }

    effect.position += (buffer_duration as f32 * effect.speed_factor) as usize;

    while effect.position > duration {
        effect.position -= duration;
    }
}

pub unsafe fn start_pitch_effect(channel: Channel, pitch_effect: *mut PitchEffect) {
    let channel_number = channel.0;

    sdl2_sys::mixer::Mix_RegisterEffect(
        channel_number as c_int,
        Some(effect_callback),
        None,
        pitch_effect as *mut c_void,
    );
}
