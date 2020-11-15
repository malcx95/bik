pub const TOTAL_NUM_LAPS: usize = 3;

// currently hardcoded to the background image size
pub const WORLD_SIZE: f32 = 3000.;
pub const DELTA_TIME: f32 = 0.01;
pub const SERVER_SLEEP_DURATION: u64 = 10;

pub const WINDOW_SIZE: f32 = 700.;
pub const PIXEL_SCALE: u32 = 4;

pub const MENU_BACKGROUND_COLOR: (u8, u8, u8) = (108, 57, 57);

pub const NAME_POS: (f32, f32) = (50., 150.);

// Steering parameters
pub const STEERING_ATTENUATION_MAX: f32 = 0.4;
pub const STEERING_RATE: f32 = 10.;
pub const STEERING_MAX: f32 = 0.5;
pub const WHEEL_DISTANCE: f32 = 20.;

pub const BIKE_SCALE: f32 = 2.0;

pub const MAX_SPEED: f32 = 600.;
pub const MAX_WALK_SPEED: f32 = 100.;
pub const ACCELERATION: f32 = 350.;

// lap info parameters
pub const LAP_POS_X: f32 = 0.5;
pub const LAP_POS_Y: f32 = 0.1;
pub const LAP_SCALE: f32 = 1.5;
pub const TIME_POS_X: f32 = 0.1;
pub const TIME_POS_Y: f32 = 0.1;
pub const TIME_PADDING: f32 = 20.;
pub const TIME_COLOR: (u8, u8, u8) = (10, 10, 10);
pub const BEST_TIME_COLOR: (u8, u8, u8) = (0, 150, 0);
pub const FINAL_LAP_COLOR: (u8, u8, u8) = (0, 150, 0);

pub const INITIAL_FUEL_LEVEL: f32 = 100.;
pub const MAX_FUEL_LEVEL: f32 = 100.;
pub const FUEL_CONSUMPTION: f32 = 5.;
pub const FUEL_PUMP_SPEED: f32 = 75.;

// powerup parameters
pub const POWERUP_DISTANCE: f32 = 100.;
pub const POWERUP_TIMEOUT: f32 = 5.;

pub const MACE_SPEED: f32 = 10.;
pub const MACE_RADIUS: f32 = 100.;

// Map parameters
pub const MAP_SCALE: f32 = 2.;
pub const CHECKPOINT_RADIUS: f32 = 300.;

// UI parameters
pub const GAUGE_BACKGROUND: (u8, u8, u8) = (10, 10, 10);
pub const GAUGE_POS_X: f32 = 0.02;
pub const GAUGE_POS_Y: f32 = 0.5;
pub const GAUGE_HEIGHT: f32 = 0.47;
pub const GAUGE_WIDTH: f32 = 0.05;
pub const GAUGE_TEXT_POS_PADDING: f32 = 0.023;

pub const RACE_COUNTDOWN_TIMER_START: f32 = 3.;

pub const PRE_RACE_PRESS_ENTER_POS_Y: f32 = 0.25;
pub const COUNTDOWN_POS_Y: f32 = 0.25;
pub const COUNTDOWN_TEXT_MAX_SIZE: f32 = 3.;
pub const COUNTDOWN_TEXT_MIN_SIZE: f32 = 2.;

pub const COLLISION_GRACE_PERIOD: f32 = 1.;
pub const COLLISION_DAMAGE: i16 = 10;
pub const COLLISION_SPEED_REDUCTION: f32 = 500.;


// Crash parameters
pub const FALLING_DURATION: f32 = 0.3;
pub const CRASH_DURATION: f32 = 1.5;
