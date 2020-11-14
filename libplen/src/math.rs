use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};
use serde_derive::{Serialize, Deserialize};
use crate::constants;

#[derive(PartialEq, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Self {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Vec2) {
        *self = *self + other;
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Self {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Self {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, scalar: f32) -> Self {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, scalar: f32) -> Self {
        Vec2 {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl Vec2 {
    pub fn from_direction(angle: f32, length: f32) -> Self {
        Vec2 {
            x: angle.cos() * length,
            y: angle.sin() * length,
        }
    }

    pub fn norm(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn normalize(self) -> Vec2 {
        self / self.norm()
    }

    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    pub fn distance_to(self, other: Self) -> f32 {
        (self - other).norm()
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
}


pub fn modulo(x: f32, div: f32) -> f32 {
    (x % div + div) % div
}


pub fn angle_diff(source_angle: f32, target_angle: f32) -> f32 {
    // From https://stackoverflow.com/a/7869457
    use std::f32::consts::PI;
    modulo(target_angle - source_angle + PI, 2. * PI) - PI
}


#[derive(Serialize, Deserialize, Clone)]
pub struct LineSegment {
    pub p1: Vec2,
    pub p2: Vec2,
}

impl LineSegment {
    pub fn new(p1: Vec2, p2: Vec2) -> Self {
        LineSegment {
            p1,
            p2,
        }
    }

    pub fn intersects(&self, v: LineSegment) -> bool {
        let a1 = self.p2.y - self.p1.y;
        let b1 = self.p1.x - self.p2.x;
        let c1 = a1 * self.p1.x + b1 * self.p1.y;

        let a2 = v.p2.y - v.p1.y;
        let b2 = v.p1.x - v.p2.x;
        let c2 = a2 * v.p1.x + b2 * v.p1.y;

        let det = a1 * b2 - a2 * b1;

        if det != 0. {
            let x = (b2 * c1 - b1 * c2) / det;
            let y = (a1 * c2 - a2 * c1) / det;

            if self.check(x, y) && v.check(x, y) && !(x == 0. && y == 0.) {
                return true;
            }
        }

        false
    }

    fn check(&self, x: f32, y: f32) -> bool {
        self.min_x() <= x && x <= self.max_x() &&
            self.min_y() <= y && y <= self.max_y()
    }

    pub fn max_x(&self) -> f32 {
        if self.p1.x >= self.p2.x {
            self.p1.x
        } else {
            self.p2.x
        }
    }

    pub fn max_y(&self) -> f32 {
        if self.p1.y >= self.p2.y {
            self.p1.y
        } else {
            self.p2.y
        }
    }

    pub fn min_x(&self) -> f32 {
        if self.p1.x <= self.p2.x {
            self.p1.x
        } else {
            self.p2.x
        }
    }

    pub fn min_y(&self) -> f32 {
        if self.p1.y <= self.p2.y {
            self.p1.y
        } else {
            self.p2.y
        }
    }
}
