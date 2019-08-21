use super::Coord2;

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

#[derive(Debug)]
pub enum State {
    Unborn,
    Alive,
    Dead,
}
