use super::Vec2;
use nalgebra_glm as glm;
use winit::VirtualKeyCode;

pub struct Camera {
    pub position: glm::TVec3<f32>,
    pub scale: f32,
    pub aspect_ratio: f32,
    ortho_projection: glm::TMat4<f32>,
    ortho_projection_amount: f32,
}

impl Camera {
    pub fn new_at_position(position: Vec2, ortho_projection_amount: f32) -> Camera {
        Camera {
            position: position.into_glm_vec3(-1.0),
            ortho_projection_amount,
            ortho_projection: Self::make_projection(ortho_projection_amount),
            aspect_ratio: 16.0 / 9.0,
            scale: 0.5,
        }
    }

    pub fn update(&mut self, input: &[VirtualKeyCode], distance: f32) {
        let right: glm::TVec3<f32> = glm::make_vec3(&[1.0, 0.0, 0.0]);
        let up: glm::TVec3<f32> = glm::make_vec3(&[0.0, 1.0, 0.0]);

        if input.iter().find(|&&key| key == VirtualKeyCode::LShift).is_some() {
            let zoom_amount: f32 = input
                .iter()
                .fold(self.ortho_projection_amount, |val, key| match *key {
                    VirtualKeyCode::W | VirtualKeyCode::Up => val + 0.1,
                    VirtualKeyCode::S | VirtualKeyCode::Down => val - 0.1,
                    _ => val,
                });

            if zoom_amount != self.ortho_projection_amount {
                self.ortho_projection = Self::make_projection(zoom_amount);
                self.ortho_projection_amount = zoom_amount;
            }
        } else {
            let mut move_vector =
                input
                    .iter()
                    .fold(glm::make_vec3(&[0.0, 0.0, 0.0]), |vec, key| match *key {
                        VirtualKeyCode::W | VirtualKeyCode::Up => vec + up,
                        VirtualKeyCode::S | VirtualKeyCode::Down => vec - up,
                        VirtualKeyCode::D | VirtualKeyCode::Right => vec + right,
                        VirtualKeyCode::A | VirtualKeyCode::Left => vec - right,
                        _ => vec,
                    });
            if move_vector != glm::zero() {
                move_vector = move_vector.normalize();
            }
            self.position += move_vector * distance;
        }
    }

    pub fn make_view_projection_mat(&self) -> glm::TMat4<f32> {
        self.ortho_projection
            * glm::look_at_lh(
                &self.position,
                &glm::make_vec3(&[self.position[0], self.position[1], 0.0]),
                &glm::make_vec3(&[0.0, 1.0, 0.0]).normalize(),
            )
    }

    pub fn display_to_world_position(&self, display_pos: Vec2, window_size: Vec2) -> Vec2 {
        let percentage_of_screen = Vec2::new(display_pos.x / window_size.x, display_pos.y / window_size.y);

        let clip_space = Vec2::new(
            percentage_of_screen.x * 2.0 - 1.0,
            percentage_of_screen.y * 2.0 - 1.0,
        );

        let reverse_view_projection =
            glm::inverse(&self.make_view_projection_mat()) * clip_space.into_glm_tmat4(0.0);

        Vec2::new(
            reverse_view_projection[12] * 1.0 / self.scale,
            reverse_view_projection[13] * 1.0 / (self.scale * self.aspect_ratio),
        )
    }

    fn make_projection(size: f32) -> glm::TMat4<f32> {
        let mut temp = glm::ortho_lh_zo(size, -size, size, -size, 0.1, 10.0);
        temp[(1, 1)] *= -1.0;
        temp
    }
}
