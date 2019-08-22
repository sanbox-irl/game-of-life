use super::{Color, Coord2};
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct Entity {
    pub coordinate: Coord2,
    pub state: State,
    matrix: Option<glm::TMat4<f32>>,
}

impl Entity {
    pub fn new(coordinate: Coord2) -> Self {
        Entity {
            coordinate,
            state: State::Unborn,
            matrix: None
        }
    }

    pub fn flip_state(&mut self) {
        let new_state = match self.state {
            State::Unborn | State::Dead => State::Alive,
            State::Alive => State::Dead,
        };

        self.state = new_state;
    }

    pub fn get_coordinate_matrix(&mut self) -> glm::TMat4<f32> {
        match self.matrix {
            Some(mat) => mat,
            None => {
                let mat = self.coordinate.into_vec2().into_glm_tmat4(0.0);
                self.matrix = Some(mat);
                mat
            }
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
