use super::{UserInput, MouseButton, Vec2};
use winit::VirtualKeyCode;

pub struct Camera {
    pub position: Vec2,
    pub scale: f32,
    pub aspect_ratio: f32,
    pub pan_speed: f32,
}

impl Camera {
    pub fn new_at_position(position: Vec2, scale: f32) -> Camera {
        let aspect_ratio = 16.0 / 9.0;
        Camera {
            position,
            aspect_ratio,
            scale,
            pan_speed: 0.05,
        }
    }

    pub fn update(&mut self, user_input: &UserInput) {
        let mut move_vector: Vec2 =
            user_input
                .kb_input
                .held_keys
                .iter()
                .fold(Vec2::ZERO, |vec, key| match *key {
                    VirtualKeyCode::W | VirtualKeyCode::Up => vec - Vec2::UP,
                    VirtualKeyCode::S | VirtualKeyCode::Down => vec + Vec2::UP,
                    VirtualKeyCode::D | VirtualKeyCode::Right => vec + Vec2::RIGHT,
                    VirtualKeyCode::A | VirtualKeyCode::Left => vec - Vec2::RIGHT,
                    _ => vec,
                });
        

        if move_vector != Vec2::ZERO {
            move_vector.normalize();
        }

        let mut pan = move_vector * self.pan_speed;

        if user_input.mouse_input.is_held(MouseButton::Middle) {
            pan -= user_input.mouse_input.mouse_delta_position() / (self.scale * 40.0);
        }

        self.position += pan;

        if user_input.mouse_input.mouse_vertical_scroll_delta != 0.0 {
            self.scale += user_input.mouse_input.mouse_vertical_scroll_delta;
            self.scale = self.scale.max(0.5);
        }
    }

    pub fn display_to_world_position(&self, display_pos: Vec2, window_size: Vec2) -> Vec2 {
        info!("--Mouse Click--");
        info!(
            "Mouse Click at display pos {} with window size {}",
            display_pos, window_size
        );
        let percentage_of_screen = Vec2::new(display_pos.x / window_size.x, display_pos.y / window_size.y);
        info!("Percentage of screen on Click is {}", percentage_of_screen);

        let clip_space = Vec2::new(
            percentage_of_screen.x * 2.0 - 1.0,
            percentage_of_screen.y * 2.0 - 1.0,
        );
        info!("Clip space is {}", clip_space);
        let mut ret = (clip_space + self.position) * self.scale;
        ret.y = ret.y / self.aspect_ratio;
        ret
    }
}
