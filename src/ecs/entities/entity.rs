use super::{Color, Coord2};

#[derive(Debug)]
pub struct Entity {
    pub coordinate: Coord2,
    pub state: State,
}

impl Entity {
    pub fn new(coordinate: Coord2) -> Self {
        Entity {
            coordinate,
            state: State::Unborn,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum State {
    Unborn,
    Alive,
    Dead,
}
impl State {
    pub fn to_color_bits(&self) -> [u32; 3] {
        match self {
            State::Alive => Color::from_u8(17, 54, 12).into_raw_u32(),
            State::Dead => Color::from_u8(47, 29, 24).into_raw_u32(),
            State::Unborn => Color::from_u8(139, 110, 101).into_raw_u32(),
        }
    }
}


