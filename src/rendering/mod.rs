pub use super::ecs::Window;
pub use super::utilities::Vec2;

macro_rules! manual_drop {
    ($this_val:expr) => {
        ManuallyDrop::into_inner(read(&$this_val))
    };
}

macro_rules! manual_new {
    ($this_val:ident) => {
        ManuallyDrop::new($this_val)
    };
}

mod buffer_bundle;
mod loaded_image;
mod pipeline_bundle;
mod renderer_commands;
mod vertex;
mod renderer;

#[derive(Debug)]
pub enum DrawingError {
    AcquireAnImageFromSwapchain,
    WaitOnFence,
    ResetFence,
    PresentIntoSwapchain,
    BufferCreationError,
}

impl std::error::Error for DrawingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for DrawingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error in Drawing {:?}", self)
    }
}

pub use super::ecs::Entity;
pub use buffer_bundle::*;
pub use loaded_image::*;
pub use pipeline_bundle::*;
pub use renderer_commands::*;
pub use vertex::*;
pub use renderer_commands::RendererCommands;
pub use renderer::TypedRenderer;
