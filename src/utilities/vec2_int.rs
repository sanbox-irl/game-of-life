use super::Vec2;
use std::convert::TryInto;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2Int {
    pub x: i32,
    pub y: i32,
}

impl Vec2Int {
    pub fn new(x: i32, y: i32) -> Self {
        Vec2Int { x, y }
    }

    pub fn magnitude(&self) -> f32 {
        ((self.x * self.x + self.y * self.y) as f32).sqrt()
    }

    // @techdebt This is a little
    pub fn normalize(&mut self) {
        let m = self.magnitude();
        self.x = (self.x as f32 / m) as i32;
        self.y = (self.y as f32 / m) as i32;
    }

    #[allow(dead_code)]
    pub fn normalized(&self) -> Self {
        let m = self.magnitude();
        let mut ret = self.clone();
        ret.x = (self.x as f32 / m) as i32;
        ret.y = (self.y as f32 / m) as i32;
        ret
    }

    #[allow(dead_code)]
    pub fn into_raw_usize(self) -> Result<(usize, usize), &'static str> {
        if self.x < 0 || self.y < 0 {
            Err("This is a negative number! Cannot cast to usize intelligently.")
        } else {
            Ok((self.x as usize, self.y as usize))
        }
    }

    #[allow(dead_code)]
    pub fn to_bits(self) -> [u32; 2] {
        [i32::try_into(self.x).unwrap(), i32::try_into(self.y).unwrap()]
    }

    pub fn clamp_components(&mut self, min_vec: &Vec2Int, max_vec: &Vec2Int) {
        self.x = self.x.max(min_vec.x).min(max_vec.x);
        self.y = self.y.max(min_vec.y).min(max_vec.y);
    }
}

impl Vec2Int {
    #[allow(dead_code)]
    pub const ZERO: Vec2Int = Vec2Int { x: 0, y: 0 };

    #[allow(dead_code)]
    pub const ONE: Vec2Int = Vec2Int { x: 1, y: 1 };

    #[allow(dead_code)]
    pub const UP: Vec2Int = Vec2Int { x: 0, y: 1 };

    #[allow(dead_code)]
    pub const RIGHT: Vec2Int = Vec2Int { x: 1, y: 0 };
}

impl Display for Vec2Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl std::ops::Add<Vec2Int> for Vec2Int {
    type Output = Vec2Int;

    fn add(self, rhs: Vec2Int) -> Vec2Int {
        Vec2Int {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign<Vec2Int> for Vec2Int {
    fn add_assign(&mut self, rhs: Vec2Int) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::Sub<Vec2Int> for Vec2Int {
    type Output = Vec2Int;

    fn sub(self, rhs: Vec2Int) -> Vec2Int {
        Vec2Int {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::SubAssign<Vec2Int> for Vec2Int {
    fn sub_assign(&mut self, rhs: Vec2Int) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::ops::Div<i32> for Vec2Int {
    type Output = Vec2Int;

    fn div(self, rhs: i32) -> Vec2Int {
        Vec2Int {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::DivAssign<i32> for Vec2Int {
    fn div_assign(&mut self, rhs: i32) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
    }
}

impl std::ops::Mul<i32> for Vec2Int {
    type Output = Vec2Int;

    fn mul(self, rhs: i32) -> Vec2Int {
        Vec2Int {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::MulAssign<i32> for Vec2Int {
    fn mul_assign(&mut self, rhs: i32) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
    }
}

impl From<[i32; 2]> for Vec2Int {
    fn from(w: [i32; 2]) -> Vec2Int {
        Vec2Int { x: w[0], y: w[1] }
    }
}

impl From<Vec2Int> for [i32; 2] {
    fn from(w: Vec2Int) -> [i32; 2] {
        [w.x, w.y]
    }
}

impl From<Vec2> for Vec2Int {
    fn from(other: Vec2) -> Vec2Int {
        Vec2Int {
            x: other.x as i32,
            y: other.y as i32,
        }
    }
}
