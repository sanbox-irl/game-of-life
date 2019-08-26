use super::Vec2;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Vec2Int {
    pub x: i32,
    pub y: i32,
}

impl Vec2Int {
    #[allow(dead_code)]
    pub fn new(x: i32, y: i32) -> Self {
        Vec2Int { x, y }
    }
    #[allow(dead_code)]
    pub fn into_vec2(self) -> Vec2 {
        Vec2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

impl fmt::Display for Vec2Int {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "[{}, {}]", self.x, self.y)
    }
}
