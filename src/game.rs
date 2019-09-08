use super::ecs::{Camera, Entity, Gameplay, Imgui, MouseButton, UiHandler, UserInput, Window};
use super::rendering::{
    DrawingError, GameWorldDrawCommands, ImGuiDrawCommands, RendererCommands, TypedRenderer,
};
use super::resources::SoundsVFX;
use super::utilities::{Time, Vec2};
use anymap::AnyMap;
use failure::Error;

const DEFAULT_SIZE: Vec2 = Vec2 { x: 1280.0, y: 720.0 };
const DEFAULT_GAME_SIZE: Vec2 = Vec2 { x: 6.0, y: 6.0 };

pub struct Game {
    pub resources: AnyMap,
    window: Window,
    user_input: UserInput,
    renderer: Option<TypedRenderer>,
    camera: Camera,
    gameplay: Gameplay,
    entities: Vec<Vec<Entity>>,
    time: Time,
}

impl Game {
    pub fn new() -> Result<Self, Error> {
        // Resources
        let mut resources = AnyMap::new();
        resources.insert(SoundsVFX::new());

        let window = Window::new(DEFAULT_SIZE)?;
        let user_input = UserInput::new();

        let renderer = TypedRenderer::typed_new(&window)?;
        let camera = Camera::new_at_position(Vec2::new(0.0, 0.0), 1.0);

        // Initialize Entities...
        let entities = Gameplay::create_game_world(DEFAULT_GAME_SIZE);
        let gameplay = Gameplay::new(&resources, DEFAULT_GAME_SIZE)?;

        Ok(Game {
            window,
            user_input,
            renderer: Some(renderer),
            entities,
            camera,
            gameplay,
            time: Time::new(),
            resources,
        })
    }

    pub fn main_loop(&mut self) -> Result<(), Error> {
        let mut dear_imgui = Imgui::new(&self.window);
        if let Some(renderer) = &mut self.renderer {
            renderer.initialize_imgui(&mut dear_imgui.imgui)?;
        };
        self.time.game_start();

        loop {
            // get input
            self.user_input.poll_events_loop(&mut self.window.events_loop);
            self.handle_window_events()?;

            // update
            dear_imgui.take_input(&mut self.user_input);
            let mut ui_frame = dear_imgui.begin_frame(&self.window);

            Imgui::make_ui(&mut ui_frame, &mut self.gameplay);
            Imgui::make_debug_ui(&ui_frame, &self.gameplay, &mut self.camera, &self.time);

            if let Some(new_entities) = self.gameplay.new_size(&mut self.entities) {
                self.entities = new_entities;
            }

            self.camera
                .update(&self.user_input, &self.window, &self.gameplay.game_size());

            // Single selection
            if self.user_input.mouse_input.is_held(MouseButton::Left) {
                let world_pos = self.camera.display_to_world_position(
                    self.user_input.mouse_input.mouse_position,
                    self.window.get_window_size(),
                );

                if let Ok(coord_pos) = world_pos.into_raw_usize() {
                    if coord_pos.0 < self.entities.len() && coord_pos.1 < self.entities[0].len() {
                        self.gameplay.select(coord_pos, &mut self.entities);
                    }
                }
            }
            self.gameplay
                .update(&self.user_input, &mut self.entities, &self.time);

            // render
            if let Err(e) = self.render(ui_frame) {
                self.renderer = None;
                break Err(e);
            }

            if self.user_input.end_requested {
                break Ok(());
            }

            self.time.end_frame();
        }
    }

    fn render(&mut self, ui_frame: UiHandler<'_>) -> Result<(), Error> {
        if let Some(renderer) = &mut self.renderer {
            let result = {
                ui_frame.prepare_draw(&self.window);
                let position = self.camera.position_scaled();

                let instructions = RendererCommands {
                    game_world_draw_commands: Some(GameWorldDrawCommands {
                        aspect_ratio: self.camera.aspect_ratio,
                        camera_position: &position,
                        camera_scale: self.camera.scale,
                        entities: &mut self.entities,
                        game_colors: &self.gameplay.game_colors,
                    }),
                    imgui_draw_commands: Some(ImGuiDrawCommands {
                        draw_data: ui_frame.ui.render(),
                        imgui_dimensions: ui_frame.size,
                    }),
                };

                renderer.draw(instructions)
            };
            match result {
                Ok(sub_optimal) => {
                    if let Some(_) = sub_optimal {
                        Game::recreate_swapchain(renderer, &self.window)
                    } else {
                        Ok(())
                    }
                }

                Err(e) => match e {
                    DrawingError::AcquireAnImageFromSwapchain | DrawingError::PresentIntoSwapchain => {
                        Game::recreate_swapchain(renderer, &self.window)
                    }

                    DrawingError::ResetFence
                    | DrawingError::WaitOnFence
                    | DrawingError::BufferCreationError => {
                        error!("Rendering Error: {:?}", e);
                        error!("Auo-restarting Renderer...");

                        self.renderer = None;
                        let ret = TypedRenderer::typed_new(&self.window);
                        match ret {
                            Ok(new_value) => {
                                self.renderer = Some(new_value);
                                debug!("Succesfully restarted Renderer!");
                                Ok(())
                            }

                            Err(e) => Err(e),
                        }
                    }
                },
            }
        } else {
            Err(format_err!(
                "Couldn't find the renderer. This should never happen."
            ))
        }
    }

    fn handle_window_events(&mut self) -> Result<(), Error> {
        if self.user_input.new_frame_size.is_some() {
            debug!("Window changed size, creating a new swapchain...");
            if let Some(renderer) = &mut self.renderer {
                let new_size = self.user_input.new_frame_size.unwrap();
                self.camera.aspect_ratio = new_size.x / new_size.y;

                info!("New Aspect Ratio is {}", self.camera.aspect_ratio);
                info!("New Size is {:?}", new_size);
                Game::recreate_swapchain(renderer, &self.window)
            } else {
                Err(format_err!(
                    "Couldn't find the renderer. This should never happen"
                ))
            }
        } else {
            Ok(())
        }
    }

    fn recreate_swapchain(renderer: &mut TypedRenderer, window: &Window) -> Result<(), Error> {
        debug!("Attempting to create a new swapchain!");
        renderer.recreate_swapchain(&window.window)
    }
}
