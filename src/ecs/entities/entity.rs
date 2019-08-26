use super::{Color, Vec2};

#[derive(Debug)]
pub struct Entity {
    pub coordinate: Vec2,
    pub state: State,
}

impl Entity {
    pub fn new(coordinate: Vec2) -> Self {
        Entity {
            coordinate,
            state: State::Unborn,
        }
    }

    pub fn flip_state(&mut self) {
        let new_state = match self.state {
            State::Unborn | State::Dead => State::Alive,
            State::Alive => State::Dead,
        };

        self.state = new_state;
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
