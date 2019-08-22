use nalgebra_glm as glm;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    #[allow(dead_code)]
    pub fn zero() -> Self {
        Vec2 { x: 0.0, y: 0.0 }
    }

    #[allow(dead_code)]
    pub fn one() -> Self {
        Vec2 { x: 1.0, y: 1.0 }
    }

    #[allow(dead_code)]
    pub fn up() -> Self {
        Vec2 { x: 0.0, y: 1.0 }
    }

    #[allow(dead_code)]
    pub fn right() -> Self {
        Vec2 { x: 1.0, y: 0.0 }
    }

    pub fn into_glm_tmat4(self, z: f32) -> glm::TMat4<f32> {
        glm::translate(&glm::identity(), &self.into_glm_vec3(z))
    }

    pub fn into_glm_vec3(self, z: f32) -> glm::TVec3<f32> {
        glm::make_vec3(&[self.x, self.y, z])
    }

    pub fn into_raw_usize(self) -> Result<(usize, usize), &'static str> {
        if self.x < 0.0 || self.y < 0.0 {
            Err("This is a negative number! Cannot case to usize intelligently.")
        } else {
            Ok((self.x as usize, self.y as usize))
        }
    }
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Coord2 {
    pub x: i32,
    pub y: i32,
}

impl Coord2 {
    pub fn new(x: i32, y: i32) -> Self {
        Coord2 { x, y }
    }

    pub fn into_vec2(self) -> Vec2 {
        Vec2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

impl Display for Coord2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "[{}, {}]", self.x, self.y)
    }
}
