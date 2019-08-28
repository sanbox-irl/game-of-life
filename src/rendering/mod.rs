pub use super::utilities::Vec2;
pub use super::ecs::Window;

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
mod pipeline_bundle;
mod loaded_image;
mod renderer;
mod vertex;

pub use super::ecs::Entity;
pub use buffer_bundle::{BufferBundle, VertexIndexPairBufferBundle};
pub use loaded_image::LoadedImage;
pub use renderer::{DrawingError, TypedRenderer};
pub use vertex::*;
pub use pipeline_bundle::PipelineBundle;
