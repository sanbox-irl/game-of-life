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
mod renderer;
mod vertex;

pub use super::ecs::Entity;
pub use buffer_bundle::BufferBundle;
pub use renderer::{DrawingError, TypedRenderer};
pub use vertex::*;