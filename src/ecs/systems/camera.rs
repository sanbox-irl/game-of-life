use nalgebra_glm as glm;
use super::Vec2;
use winit::VirtualKeyCode;

pub struct Camera {
    pub position: glm::TVec3<f32>,
    ortho_projection: glm::TMat4<f32>,
}

impl Camera {
    pub fn new_at_position(position: Vec2, ortho_projection: glm::TMat4<f32>) -> Camera {
        Camera {
            position: position.into_glm_vec3(-1.0),
            ortho_projection,
        }
    }

    pub fn update_position(&mut self, input: &[VirtualKeyCode], distance: f32) {
        let right: glm::TVec3<f32> = glm::make_vec3(&[1.0, 0.0, 0.0]);
        let up: glm::TVec3<f32> = glm::make_vec3(&[0.0, 1.0, 0.0]);

        let mut move_vector = input
            .iter()
            .fold(glm::make_vec3(&[0.0, 0.0, 0.0]), |vec, key| match *key {
                VirtualKeyCode::W | VirtualKeyCode::Up => vec - up,
                VirtualKeyCode::S | VirtualKeyCode::Down => vec + up,
                VirtualKeyCode::D | VirtualKeyCode::Right => vec + right,
                VirtualKeyCode::A | VirtualKeyCode::Left => vec - right,
                _ => vec,
            });
        if move_vector != glm::zero() {
            move_vector = move_vector.normalize();
        }
        self.position += move_vector * distance;
    }

    pub fn make_view_matrix(&self) -> glm::TMat4<f32> {
        glm::look_at_lh(
            &self.position,
            &glm::make_vec3(&[self.position[0], self.position[1], 0.0]),
            &glm::make_vec3(&[0.0, 1.0, 0.0]).normalize(),
        ) * self.ortho_projection
    }
}
