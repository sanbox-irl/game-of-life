use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&mut self) {
        let m = self.magnitude();
        self.x = self.x / m;
        self.y = self.y / m;
    }

    #[allow(dead_code)]
    pub fn normalized(&self) -> Self {
        let m = self.magnitude();
        self.clone() / m
    }

    #[allow(dead_code)]
    pub fn into_raw_usize(self) -> Result<(usize, usize), &'static str> {
        if self.x < 0.0 || self.y < 0.0 {
            Err("This is a negative number! Cannot case to usize intelligently.")
        } else {
            Ok((self.x as usize, self.y as usize))
        }
    }

    #[allow(dead_code)]
    pub fn to_bits(self) -> [u32; 2] {
        [self.x.to_bits(), self.y.to_bits()]
    }
}

impl Vec2 {
    #[allow(dead_code)]
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    #[allow(dead_code)]
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };

    #[allow(dead_code)]
    pub const UP: Vec2 = Vec2 { x: 0.0, y: 1.0 };

    #[allow(dead_code)]
    pub const RIGHT: Vec2 = Vec2 { x: 1.0, y: 0.0 };
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl std::ops::Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::SubAssign<Vec2> for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::ops::Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(w: [f32; 2]) -> Vec2 {
        Vec2 { x: w[0], y: w[1] }
    }
}

impl From<Vec2> for [f32; 2] {
    fn from(w: Vec2) -> [f32; 2] {
        [w.x, w.y]
    }
}
