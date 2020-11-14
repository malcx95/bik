// currently hardcoded to the background image size
pub const WORLD_SIZE: f32 = 3000.;
pub const DELTA_TIME: f32 = 0.01;
pub const SERVER_SLEEP_DURATION: u64 = 10;

pub const WINDOW_SIZE: f32 = 700.;

pub const MENU_BACKGROUND_COLOR: (u8, u8, u8) = (108, 57, 57);

pub const NAME_POS: (f32, f32) = (50., 150.);

// Steering parameters
pub const STEERING_ATTENUATION_MAX: f32 = 0.2;
pub const STEERING_RATE: f32 = 3.;
pub const STEERING_MAX: f32 = 0.5;
pub const WHEEL_DISTANCE: f32 = 50.;

pub const BIKE_SCALE: f32 = 0.5;

pub const MAX_SPEED: f32 = 350.;
pub const MAX_BACKWARD_SPEED: f32 = 50.;
pub const ACCELERATION: f32 = 350.;

// lap info parameters
pub const LAP_POS: (f32, f32) = (10., 10.);

pub const INITIAL_FUEL_LEVEL: f32 = 100.;
pub const MAX_FUEL_LEVEL: f32 = 100.;
pub const FUEL_CONSUMPTION: f32 = 1.;

pub const POWERUP_DISTANCE: f32 = 100.;
