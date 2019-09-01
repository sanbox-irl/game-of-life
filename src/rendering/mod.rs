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
mod renderer;
mod renderer_errors;
mod renderer_commands;
mod vertex;

pub use super::ecs::Entity;
pub use renderer_errors::*;
pub use buffer_bundle::*;
pub use loaded_image::*;
pub use pipeline_bundle::*;
pub use renderer::TypedRenderer;
pub use renderer_commands::RendererCommands;
pub use renderer_commands::*;
pub use vertex::*;
