use super::{MouseButton, UserInput, Vec2, Window as WinitWindow, Vec2Int};
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

    pub fn update(&mut self, user_input: &UserInput, winit_window: &WinitWindow, game_size: &Vec2Int) {
        let mut move_vector: Vec2 =
            user_input
                .kb_input
                .held_keys
                .iter()
                .fold(Vec2::ZERO, |vec, key| match *key {
                    VirtualKeyCode::W | VirtualKeyCode::Up => vec + Vec2::UP,
                    VirtualKeyCode::S | VirtualKeyCode::Down => vec - Vec2::UP,
                    VirtualKeyCode::D | VirtualKeyCode::Right => vec - Vec2::RIGHT,
                    VirtualKeyCode::A | VirtualKeyCode::Left => vec + Vec2::RIGHT,
                    _ => vec,
                });

        if move_vector != Vec2::ZERO {
            move_vector.normalize();
        }

        let mut pan = move_vector * (self.scale * 0.5) * self.pan_speed;

        if user_input.mouse_input.is_held(MouseButton::Middle) {
            let window_size = winit_window.get_window_size();
            let old_pos = self.display_to_world_position(
                user_input.mouse_input.mouse_position_last_frame,
                window_size.clone(),
            );
            let new_pos = self.display_to_world_position(user_input.mouse_input.mouse_position, window_size);

            let mut ret = (new_pos - old_pos) * self.scale;
            ret.y *= self.aspect_ratio;

            pan += ret / self.scale;
        }

        self.position -= pan;
        let mut size: Vec2 = game_size.clone().into();
        size.y *= self.aspect_ratio;
        self.position.clamp_components(&Vec2::ZERO, &size);

        if user_input.mouse_input.mouse_vertical_scroll_delta != 0.0 {
            self.scale += user_input.mouse_input.mouse_vertical_scroll_delta;
            self.scale = self.scale.min(size.x * 2.0).max(0.5);
        }
    }

    pub fn position_scaled(&self) -> Vec2 {
        self.position / self.scale
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
        let mut ret = clip_space * self.scale + self.position;
        ret.y = ret.y / self.aspect_ratio;
        info!("Real Position is {}", ret);
        ret
    }
}
