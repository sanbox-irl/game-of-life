use super::Vec2;
use winit::{dpi::LogicalSize, CreationError, EventsLoop, Window as WinitWindow, WindowBuilder};

const WINDOW_NAME: &'static str = "Game of Life by Jack Spira";

pub struct Window {
    pub name: &'static str,
    pub events_loop: EventsLoop,
    pub window: WinitWindow,
}

impl Window {
    pub fn new(size: Vec2) -> Result<Self, CreationError> {
        let events_loop = EventsLoop::new();
        let output = WindowBuilder::new()
            .with_title(WINDOW_NAME)
            .with_dimensions(LogicalSize {
                width: size.x as f64,
                height: size.y as f64,
            })
            .build(&events_loop);

        output.map(|window| Self {
            events_loop,
            window,
            name: WINDOW_NAME,
        })
    }

    pub fn get_window_size(&self) -> Vec2 {
        let window_client_area = self
            .window
            .get_inner_size()
            .unwrap()
            .to_physical(self.window.get_hidpi_factor());

        Vec2::new(window_client_area.width as f32, window_client_area.height as f32)
    }
}
