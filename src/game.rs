use super::ecs::{Camera, Entity, UserInput, Window};
use super::rendering::{DrawingError, TypedRenderer};
use super::utilities::{Coord2, Vec2};
use nalgebra_glm as glm;

const DEFAULT_SIZE: Vec2 = Vec2 { x: 1280.0, y: 720.0 };

const ARRAY_SIZE: Coord2 = Coord2 { x: 10, y: 10 };

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

        let camera = Camera::new_at_position(Vec2::new(0.0, 0.0), {
            let mut temp = glm::ortho_lh_zo(-5.0, 5.0, -5.0, 5.0, 0.1, 10.0);
            temp[(1, 1)] *= -1.0;
            temp
        });
        
        // Initialize Entities...
        let mut entities = vec![];
        for y in 0..ARRAY_SIZE.y {
            let mut this_vec = vec![];
            for x in 0..ARRAY_SIZE.x {
                this_vec.push(Entity::new(Coord2::new(x, y)));
            }
            entities.push(this_vec);
        }

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
        loop {
            // get input
            self.user_input.poll_events_loop(&mut self.window.events_loop);
            if self.handle_window_events() == false {
                break false;
            }

            // update
            self.camera.update_position(&self.user_input.held_keys, 0.05);

            // render
            if self.render() == false {
                self.renderer = None;
                break false;
            }

            if self.user_input.end_requested {
                break true;
            }
        }
    }

    fn render(&mut self) -> bool {
        if let Some(renderer) = &mut self.renderer {
            match renderer.draw_quad_frame(&self.entities, &self.camera.make_view_matrix()) {
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
