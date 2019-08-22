use super::Vec2;
use arrayvec::ArrayVec;
use winit::{
    DeviceEvent, ElementState, Event, EventsLoop, KeyboardInput as WinitKeyboardInput,
    MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent, dpi::LogicalPosition
};

#[derive(Debug)]
pub struct UserInput {
    pub end_requested: bool,
    pub new_frame_size: Option<Vec2>,
    pub mouse_input: MouseInput,
    pub kb_input: KeyboardInput,
}

impl UserInput {
    pub fn new() -> Self {
        UserInput {
            end_requested: false,
            new_frame_size: None,
            mouse_input: MouseInput::default(),
            kb_input: KeyboardInput::default(),
        }
    }

    pub fn poll_events_loop(&mut self, events_loop: &mut EventsLoop) {
        let keys_pressed_last_frame = self.kb_input.pressed_keys.clone();
        let mouse_button_clicked_last_frame = self.mouse_input.mouse_pressed;
        self.clear_input();

        events_loop.poll_events(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                self.end_requested = true;
                debug!("End was requested!");
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(logical),
                ..
            } => {
                self.new_frame_size = Some(Vec2::new(logical.width as f32, logical.height as f32));
                debug!("Our new frame size is {:?}", self.new_frame_size);
            }

            Event::DeviceEvent {
                event:
                    DeviceEvent::Key(WinitKeyboardInput {
                        virtual_keycode: Some(code),
                        state,
                        ..
                    }),
                ..
            } => self.record_input(state, code, &keys_pressed_last_frame),

            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                self.mouse_input.mouse_position = Vec2::new(position.x as f32, position.y as f32);
            }

            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                if mouse_button_clicked_last_frame == false {
                    self.mouse_input.mouse_pressed = true;
                    self.mouse_input.mouse_held = true;
                }
            }

            Event::WindowEvent {
                event: WindowEvent::MouseWheel {
                    delta: scroll_delta, ..
                },
                ..
            } => match scroll_delta {
                MouseScrollDelta::PixelDelta(LogicalPosition {
                    x: _,
                    y: vertical_move,
                }) => {
                    self.mouse_input.mouse_vertical_scroll_delta = -vertical_move as f32;
                }

                MouseScrollDelta::LineDelta(_, vertical_move) => {
                    self.mouse_input.mouse_vertical_scroll_delta = -vertical_move;
                }
            },

            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                if self.mouse_input.mouse_pressed || self.mouse_input.mouse_held {
                    self.mouse_input.mouse_pressed = false;
                    self.mouse_input.mouse_held = false;

                    self.mouse_input.mouse_released = true;
                }
            }

            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            WinitKeyboardInput {
                                state,
                                virtual_keycode: Some(code),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if cfg!(feature = "metal") {
                    self.record_input(state, code, &keys_pressed_last_frame);
                }
            }
            _ => (),
        });
    }

    pub fn clear_input(&mut self) {
        self.end_requested = false;
        self.new_frame_size = None;
        self.mouse_input.clear();
        self.kb_input.clear();
    }

    pub fn record_input(
        &mut self,
        element_state: ElementState,
        code: VirtualKeyCode,
        last_frame_pressed: &[VirtualKeyCode],
    ) {
        match element_state {
            ElementState::Pressed => {
                if let None = last_frame_pressed.iter().position(|&pos| pos == code) {
                    if let None = self.kb_input.held_keys.iter().position(|&pos| pos == code) {
                        trace!("Pressed key {:?}", code);
                        self.kb_input.pressed_keys.push(code);
                        self.kb_input.held_keys.push(code);
                    }
                }
            }

            ElementState::Released => {
                if let Some(vk_pos) = self.kb_input.held_keys.iter().position(|&item| item == code) {
                    self.kb_input.held_keys.remove(vk_pos);
                    self.kb_input.released_keys.push(code);
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct MouseInput {
    pub mouse_position: Vec2,
    pub mouse_vertical_scroll_delta: f32,
    pub mouse_pressed: bool,
    pub mouse_held: bool,
    pub mouse_released: bool,
}

impl MouseInput {
    pub fn clear(&mut self) {
        self.mouse_pressed = false;
        self.mouse_held = false;
        self.mouse_released = false;
        self.mouse_vertical_scroll_delta = 0.0;
    }
}

#[derive(Debug, Default)]
pub struct KeyboardInput {
    pub pressed_keys: ArrayVec<[VirtualKeyCode; 10]>,
    pub held_keys: ArrayVec<[VirtualKeyCode; 10]>,
    pub released_keys: ArrayVec<[VirtualKeyCode; 10]>,
}

impl KeyboardInput {
    pub fn clear(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
    }
}
