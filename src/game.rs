use super::ecs::{rule_setter, Camera, Entity, State, UserInput, Window};
use super::rendering::{DrawingError, TypedRenderer};
use super::utilities::{Coord2, Vec2};
use winit::VirtualKeyCode;
use std::time::Instant;

const DEFAULT_SIZE: Vec2 = Vec2 { x: 1920.0, y: 1080.0 };
const ARRAY_SIZE: Coord2 = Coord2 { x: 100, y: 100 };

//  JACK WE'RE HERE. WHY DOESN'T 1-1 APPEAR BUT 2-2+ DOES APPEAR?
// DEBUG AND CHECK THE MATRICES

pub struct Game {
    window: Window,
    user_input: UserInput,
    renderer: Option<TypedRenderer>,
    camera: Camera,
    entities: Vec<Vec<Entity>>,
}

impl Game {
    pub fn new() -> Result<Self, &'static str> {
        let window = Window::new(DEFAULT_SIZE).map_err(|_| "Couldn't create the window!")?;
        let user_input = UserInput::new();

        let renderer = TypedRenderer::typed_new(&window)?;
        let camera = Camera::new_at_position(Vec2::new(0.0, 0.0), 5);

        // Initialize Entities...
        let mut entities = vec![];
        for x in 0..ARRAY_SIZE.y {
            let mut this_vec = vec![];
            for y in 0..ARRAY_SIZE.x {
                this_vec.push(Entity::new(Coord2::new(x, y)));
            }
            entities.push(this_vec);
        }

        // Basic test:
        entities[0][0].state = State::Dead;
        // entities[4][6].state = State::Alive;
        // entities[5][6].state = State::Alive;
        // entities[6][6].state = State::Alive;

        // entities[4][5].state = State::Alive;
        // entities[6][5].state = State::Alive;

        // entities[4][4].state = State::Alive;
        // entities[5][4].state = State::Alive;
        // entities[6][4].state = State::Alive;

        info!("Entities: {:#?}", entities);

        Ok(Game {
            window,
            user_input,
            renderer: Some(renderer),
            entities,
            camera,
        })
    }

    pub fn main_loop(&mut self) -> bool {
        let mut time = Instant::now();

        loop {
            // get input
            self.user_input.poll_events_loop(&mut self.window.events_loop);
            if self.handle_window_events() == false {
                break false;
            }

            // update
            self.camera.update(
                &self.user_input.kb_input.held_keys,
                0.05,
                self.user_input.mouse_input.mouse_vertical_scroll_delta,
            );

            if self.user_input.mouse_input.mouse_pressed {
                let world_pos = self.camera.display_to_world_position(
                    self.user_input.mouse_input.mouse_position,
                    self.window.get_window_size(),
                );

                if let Ok(coord_pos) = world_pos.into_raw_usize() {
                    if coord_pos.0 < self.entities.len() && coord_pos.1 < self.entities[0].len() {
                        self.entities[coord_pos.0][coord_pos.1].flip_state();
                    }
                }
            }

            if self
                .user_input
                .kb_input
                .pressed_keys
                .iter()
                .find(|&&key| key == VirtualKeyCode::Return)
                .is_some()
            {
                rule_setter::set_rules(&mut self.entities);
            }

            // render
            if self.render() == false {
                self.renderer = None;
                break false;
            }

            {
                let new_time = Instant::now();
                let difference = new_time.duration_since(time);
                println!("Time difference is {}", difference.as_secs() as f32 + difference.subsec_nanos() as f32 * 1e-9);
                time = new_time;
            }

            if self.user_input.end_requested {
                break true;
            }
        }
    }

    fn render(&mut self) -> bool {
        if let Some(renderer) = &mut self.renderer {
            match renderer.draw_quad_frame(
                &mut self.entities,
                &self.camera.make_view_projection_mat(),
            ) {
                Ok(sub_optimal) => {
                    if let Some(_) = sub_optimal {
                        Game::recreate_swapchain(renderer, &self.window)
                    } else {
                        true
                    }
                }

                Err(e) => match e {
                    DrawingError::AcquireAnImageFromSwapchain | DrawingError::PresentIntoSwapchain => {
                        Game::recreate_swapchain(renderer, &self.window)
                    }

                    DrawingError::ResetFence | DrawingError::WaitOnFence => {
                        error!("Rendering Error: {:?}", e);
                        error!("Auo-restarting Renderer...");

                        self.renderer = None;
                        let ret = TypedRenderer::typed_new(&self.window);

                        match ret {
                            Ok(new_value) => {
                                self.renderer = Some(new_value);
                                debug!("Succesfully restarted Renderer!");
                                true
                            }

                            Err(_) => {
                                error!("Couldn't recover from error.");
                                false
                            }
                        }
                    }
                },
            }
        } else {
            false
        }
    }

    fn handle_window_events(&mut self) -> bool {
        if self.user_input.new_frame_size.is_some() {
            debug!("Window changed size, creating a new swapchain...");
            if let Some(renderer) = &mut self.renderer {
                let new_size = self.user_input.new_frame_size.unwrap();
                self.camera.aspect_ratio = new_size.x / new_size.y;
                self.camera.scale = 540.0 / new_size.y;

                info!("New Aspect Ratio is {}", self.camera.aspect_ratio);
                info!("New Size is {:?}", new_size);
                info!("New Scale is {}", self.camera.scale);
                Game::recreate_swapchain(renderer, &self.window)
            } else {
                false
            }
        } else {
            true
        }
    }

    fn recreate_swapchain(renderer: &mut TypedRenderer, window: &Window) -> bool {
        debug!("Attempting to create a new swapchain!");
        if let Err(e) = renderer.recreate_swapchain(&window.window) {
            error!("{}", e);
            error!("Couldn't recreate the swapchain. Exiting...");
            false
        } else {
            true
        }
    }
}
