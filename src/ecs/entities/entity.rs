use super::{Color, Vec2};

#[derive(Debug)]
pub struct Entity {
    pub position: Vec2,
    pub state: State,
    pub selection_borders: Borders,
}

impl Entity {
    pub fn new(position: Vec2) -> Self {
        Entity {
            position,
            state: State::Unborn,
            selection_borders: Borders::empty(),
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

bitflags! {
    pub struct Borders: u32 {
        const RIGHT =   0b00000001;
        const UP    =   0b00000010; 
        const LEFT  =   0b00000100; 
        const DOWN  =   0b00001000; 
    }
}