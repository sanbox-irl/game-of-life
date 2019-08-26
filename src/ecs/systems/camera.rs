use super::Vec2;
use winit::VirtualKeyCode;

pub struct Camera {
    pub position: Vec2,
    pub scale: f32,
    pub aspect_ratio: f32,
    pub ortho_projection_amount: f32,
}

impl Camera {
    pub fn new_at_position(position: Vec2, ortho_projection_amount: u32) -> Camera {
        let ortho_projection_amount = ortho_projection_amount as f32;
        let aspect_ratio = 16.0 / 9.0;
        let scale = 0.5;
        Camera {
            position,
            ortho_projection_amount,
            aspect_ratio,
            scale,
        }
    }

    pub fn update(&mut self, input: &[VirtualKeyCode], distance: f32, scroll_delta: f32) {
        if input.iter().find(|&&key| key == VirtualKeyCode::LShift).is_some() {
            let zoom_amount: f32 = input
                .iter()
                .fold(self.ortho_projection_amount, |val, key| match *key {
                    VirtualKeyCode::W | VirtualKeyCode::Up => val + 0.1,
                    VirtualKeyCode::S | VirtualKeyCode::Down => val - 0.1,
                    _ => val,
                });

            if zoom_amount != self.ortho_projection_amount {
                self.ortho_projection_amount = zoom_amount;
            }
        } else {
            let mut move_vector: Vec2 = input.iter().fold(Vec2::ZERO, |vec, key| match *key {
                VirtualKeyCode::W | VirtualKeyCode::Up => vec - Vec2::UP,
                VirtualKeyCode::S | VirtualKeyCode::Down => vec + Vec2::UP,
                VirtualKeyCode::D | VirtualKeyCode::Right => vec + Vec2::RIGHT,
                VirtualKeyCode::A | VirtualKeyCode::Left => vec - Vec2::RIGHT,
                _ => vec,
            });
            if move_vector != Vec2::ZERO {
                move_vector.normalize();
            }

            self.position += move_vector * distance;
        }

        if scroll_delta != 0.0 {
            self.ortho_projection_amount = (self.ortho_projection_amount + scroll_delta).max(0.5);
        }
    }

    pub fn display_to_world_position(&self, display_pos: Vec2, window_size: Vec2) -> Vec2 {
        let percentage_of_screen = Vec2::new(display_pos.x / window_size.x, display_pos.y / window_size.y);

        let clip_space = Vec2::new(
            percentage_of_screen.x * 2.0 - 1.0,
            percentage_of_screen.y * 2.0 - 1.0,
        );
        let mut ret = (clip_space + self.position) * self.ortho_projection_amount;
        ret.y = ret.y / self.aspect_ratio;
        ret
    }
}
