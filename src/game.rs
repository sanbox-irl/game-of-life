use super::ecs::{Camera, Entity, UserInput, Window};
use super::rendering::{DrawingError, TypedRenderer};
use super::utilities::Vec2;
use arrayvec::ArrayVec;
use nalgebra_glm as glm;

const DEFAULT_SIZE: Vec2 = Vec2 { x: 1280.0, y: 720.0 };

pub struct Game {
    window: Window,
    user_input: UserInput,
    renderer: Option<TypedRenderer>,
    camera: Camera,
    entities: ArrayVec<[Entity; 1024]>,
}

impl Game {
    pub fn new() -> Result<Self, &'static str> {
        let window = Window::new(DEFAULT_SIZE).map_err(|_| "Couldn't create the window!")?;
        let user_input = UserInput::new();

        let renderer = TypedRenderer::typed_new(&window)?;
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
            renderer: Some(renderer),
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
                        if let Err(e) = renderer.recreate_swapchain(&self.window.window) {
                            error!("{}", e);
                            error!("Couldn't recreate the swapchain. Exiting...");
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                }

                Err(e) => match e {
                    DrawingError::AcquireAnImageFromSwapchain | DrawingError::PresentIntoSwapchain => {
                        debug!("Creating new swapchain!");
                        if let Err(e) = renderer.recreate_swapchain(&self.window.window) {
                            error!("{}", e);
                            error!("Couldn't recreate the swapchain. Exiting...");
                            false
                        } else {
                            true
                        }
                    }

                    DrawingError::ResetFence | DrawingError::WaitOnFence => {
                        error!("Rendering Error: {:?}", e);
                        info!("Auo-restarting Renderer...");

                        self.renderer = None;
                        let ret = TypedRenderer::typed_new(&self.window);

                        match ret {
                            Ok(new_value) => {
                                self.renderer = Some(new_value);
                                info!("Succesfully restarted Renderer!");
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
}
