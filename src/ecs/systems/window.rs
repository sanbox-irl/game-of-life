use winit::{dpi::LogicalSize, EventsLoop, Window as WinitWindow, WindowBuilder, CreationError};
use super::Vec2;

pub struct Window {
    pub events_loop: EventsLoop,
    pub window: WinitWindow,
}

impl Window {
    pub fn new(title: &str, size: Vec2) -> Result<Self, CreationError> {
        let events_loop = EventsLoop::new();
        let output = WindowBuilder::new()
            .with_title(title)
            .with_dimensions(LogicalSize {
                width: size.x as f64,
                height: size.y as f64,
            })
            .build(&events_loop);

        output.map(|window| Self {
            events_loop,
            window,
        })
    }
}
