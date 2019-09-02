use super::Vec2;
use arrayvec::ArrayVec;
use winit::{
    dpi::LogicalPosition, DeviceEvent, ElementState, Event, EventsLoop, KeyboardInput as WinitKeyboardInput,
    MouseButton as WinitMouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
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
                        button,
                        ..
                    },
                ..
            } => {
                let this_button = match button {
                    WinitMouseButton::Left => 0,
                    WinitMouseButton::Right => 1,
                    WinitMouseButton::Middle => 2,
                    WinitMouseButton::Other(num) => num as usize,
                };

                if mouse_button_clicked_last_frame[this_button] == false {
                    self.mouse_input.mouse_pressed[this_button] = true;
                    self.mouse_input.mouse_held[this_button] = true;
                }
            }

            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button,
                        ..
                    },
                ..
            } => {
                let this_button = match button {
                    WinitMouseButton::Left => 0,
                    WinitMouseButton::Right => 1,
                    WinitMouseButton::Middle => 2,
                    WinitMouseButton::Other(num) => num as usize,
                };

                if self.mouse_input.mouse_pressed[this_button] || self.mouse_input.mouse_held[this_button] {
                    self.mouse_input.mouse_pressed[this_button] = false;
                    self.mouse_input.mouse_held[this_button] = false;

                    self.mouse_input.mouse_released[this_button] = true;
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
    pub mouse_pressed: [bool; 5],
    pub mouse_held: [bool; 5],
    pub mouse_released: [bool; 5],
}

impl MouseInput {
    pub fn clear(&mut self) {
        for elem in self.mouse_pressed.iter_mut() {
            *elem = false;
        }
        for elem in self.mouse_released.iter_mut() {
            *elem = false;
        }
        self.mouse_vertical_scroll_delta = 0.0;
    }


    pub fn is_pressed(&self, mouse_button: MouseButton) -> bool {
        let index: usize = mouse_button.into();
        self.mouse_pressed[index]
    }

    pub fn is_held(&self, mouse_button: MouseButton) -> bool {
        let index: usize = mouse_button.into();
        self.mouse_held[index]
    }

    pub fn is_released(&self, mouse_button: MouseButton) -> bool {
        let index: usize = mouse_button.into();
        self.mouse_released[index]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Generic(usize),
}

impl From<MouseButton> for usize {
    fn from(w: MouseButton) -> usize {
        match w {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Generic(index) => index,
        }
    }
}

#[derive(Debug, Default)]
pub struct KeyboardInput {
    pub pressed_keys: ArrayVec<[VirtualKeyCode; 10]>,
    pub held_keys: ArrayVec<[VirtualKeyCode; 10]>,
    pub released_keys: ArrayVec<[VirtualKeyCode; 10]>,
}

macro_rules! quick_find {
    ($iterable:expr, $target:expr) => {
        $iterable.iter().find(|&&x| x == $target)
    };
}

impl KeyboardInput {
    pub fn clear(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
    }

    #[allow(dead_code)]
    pub fn is_pressed(&mut self, target_keycode: VirtualKeyCode) -> bool {
        quick_find!(self.pressed_keys, target_keycode).is_some()
    }

    #[allow(dead_code)]
    pub fn is_held(&mut self, target_keycode: VirtualKeyCode) -> bool {
        quick_find!(self.held_keys, target_keycode).is_some()
    }

    #[allow(dead_code)]
    pub fn is_released(&mut self, target_keycode: VirtualKeyCode) -> bool {
        quick_find!(self.released_keys, target_keycode).is_some()
    }
}
