use super::Vec2;
use arrayvec::ArrayVec;
use winit::{DeviceEvent, ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, WindowEvent};

#[derive(Debug)]
pub struct UserInput {
    pub end_requested: bool,
    pub new_frame_size: Option<Vec2>,
    pub new_mouse_position: Option<Vec2>,
    pub pressed_keys: ArrayVec<[VirtualKeyCode; 10]>,
    pub held_keys: ArrayVec<[VirtualKeyCode; 10]>,
    pub released_keys: ArrayVec<[VirtualKeyCode; 10]>,
}

impl UserInput {
    pub fn new() -> Self {
        UserInput {
            end_requested: false,
            new_frame_size: None,
            new_mouse_position: None,
            pressed_keys: ArrayVec::new(),
            held_keys: ArrayVec::new(),
            released_keys: ArrayVec::new(),
        }
    }

    pub fn poll_events_loop(&mut self, events_loop: &mut EventsLoop) {
        let keys_pressed_last_frame = self.pressed_keys.clone();
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
                    DeviceEvent::Key(KeyboardInput {
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
                self.new_mouse_position = Some(Vec2::new(position.x as f32, position.y as f32));
            }

            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
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
        self.new_mouse_position = None;
        self.pressed_keys.clear();
        self.released_keys.clear();
    }

    pub fn record_input(&mut self, element_state: ElementState, code: VirtualKeyCode, last_frame_pressed: &[VirtualKeyCode]) {
        match element_state {
            ElementState::Pressed => {
                if let None = last_frame_pressed.iter().position(|&pos| pos == code) {
                    if let None = self.held_keys.iter().position(|&pos| pos == code) {
                        trace!("Pressed key {:?}", code);
                        self.pressed_keys.push(code);
                        self.held_keys.push(code);
                    }
                }
            }

            ElementState::Released => {
                if let Some(vk_pos) = self.held_keys.iter().position(|&item| item == code) {
                    self.held_keys.remove(vk_pos);
                    self.released_keys.push(code);
                }
            }
        }
    }
}
