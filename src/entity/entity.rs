use super::Vec2;

#[derive(Debug)]
pub struct Entity {
    pub position: Vec2,
    pub state: State,
}

impl Entity {
    pub fn new(position: Vec2) -> Self {
        Entity {
            position,
            state: State::Unborn,
        }
    }

    pub fn flip_state(&mut self) -> State {
        let new_state = match self.state {
            State::Unborn | State::Dead => State::Alive,
            State::Alive => State::Dead,
        };

        self.state = new_state;
        new_state
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum State {
    Unborn,
    Alive,
    Dead,
}