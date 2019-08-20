use super::ecs::{Camera, Entity, UserInput, Window};
use super::gfx_hal::window::Suboptimal;
use super::rendering::{DrawingError, TypedRenderer};
use super::utilities::Vec2;
use arrayvec::ArrayVec;
use nalgebra_glm as glm;

const WINDOW_NAME: &'static str = "Game of Life by Jack Spira";
const DEFAULT_SIZE: Vec2 = Vec2 { x: 1280.0, y: 720.0 };

pub struct Game {
    window: Window,
    user_input: UserInput,
    renderer: TypedRenderer,
    camera: Camera,
    entities: ArrayVec<[Entity; 1024]>,
}

impl Game {
    pub fn new() -> Result<Self, &'static str> {
        let window = Window::new(WINDOW_NAME, DEFAULT_SIZE).map_err(|_| "Couldn't create the window!")?;
        let user_input = UserInput::new();

        let renderer = TypedRenderer::typed_new(&window.window, WINDOW_NAME)?;
        let mut entities = ArrayVec::new();
        entities.push(Entity {
            position: Vec2::new(0.0, 0.0),
        });
        let camera = Camera::new_at_position(Vec2::new(0.0, 0.0), {
            let mut temp = glm::ortho_lh_zo(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0);
            temp[(1, 1)] *= -1.0;
            temp
        });

        Ok(Game {
            window,
            user_input,
            renderer,
            entities,
            camera,
        })
    }

    pub fn main_loop(&mut self) -> bool {
        loop {
            self.user_input.poll_events_loop(&mut self.window.events_loop);

            // update
            self.camera.update_position(&self.user_input.held_keys, 0.05);

            // render
            self.render().unwrap();

            if self.user_input.end_requested {
                break true;
            }
        }
    }

    fn render(&mut self) -> Result<Option<Suboptimal>, DrawingError> {
        self.renderer
            .draw_quad_frame(&self.entities, &self.camera.make_view_matrix())
    }
}
