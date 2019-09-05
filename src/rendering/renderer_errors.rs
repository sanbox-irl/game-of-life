#[allow(unused_macros)]
macro_rules! quick_from {
    ($our_type:ty, $our_member:expr, $target_type:ty) => {
        impl From<$target_type> for $our_type {
            fn from(error: $target_type) -> Self {
                $our_member(error)
            }
        }
    };
}

#[derive(Debug, Fail)]
pub enum DrawingError {
    AcquireAnImageFromSwapchain,
    WaitOnFence,
    ResetFence,
    PresentIntoSwapchain,
    BufferCreationError,
}

impl std::fmt::Display for DrawingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error in Drawing {:?}", self)
    }
}

use gfx_hal::device::{OomOrDeviceLost, OutOfMemory, ShaderError};
use gfx_hal::error::DeviceCreationError;
#[derive(Debug, Fail)]
pub enum RendererCreationError {
    GraphicalAdapter,
    FindQueueFamily,
    OpenPhysicalAdapter(#[cause] DeviceCreationError),
    OwnershipQueueGroup,
    FindCommandQueue,
    PresentMode,
    EmptyFormatList,
    WindowExist,
    SurfaceColor,
    Swapchain(#[cause] gfx_hal::window::CreationError),
    Fence(#[cause] OutOfMemory),
    ImageAvailableSemaphore(#[cause] OutOfMemory),
    RenderFinishedSemaphore(#[cause] OutOfMemory),
    RenderPassCreation(#[cause] OutOfMemory),
    ImageViews(#[cause] gfx_hal::image::ViewError),
    FrameBuffers(#[cause] OutOfMemory),
    CommandPool(#[cause] OutOfMemory),
}

impl std::fmt::Display for RendererCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write: String = match self {
            RendererCreationError::GraphicalAdapter => "Couldn't find a graphical adapter!".to_owned(),
            RendererCreationError::FindQueueFamily => {
                "Couldn't find a queue family with graphics!".to_owned()
            }
            RendererCreationError::OpenPhysicalAdapter(e) => {
                format!("Couldn't open the Physical Device => {}", e)
            }
            RendererCreationError::OwnershipQueueGroup => {
                "Couldn't take ownership of the QueueGroup".to_owned()
            }
            RendererCreationError::FindCommandQueue => {
                "The QueueGroup did not have any CommandQueues available".to_owned()
            }
            RendererCreationError::PresentMode => "No PresentMode values specified!".to_owned(),
            RendererCreationError::EmptyFormatList => "Preferred format list was empty!".to_owned(),
            RendererCreationError::WindowExist => "Window doesn't exist!".to_owned(),
            RendererCreationError::SurfaceColor => {
                "the Surface isn't capable of supporting color!".to_owned()
            }
            RendererCreationError::Swapchain(e) => format!("Failed to create the swapchain! => {}", e),
            RendererCreationError::Fence(e) => format!("Couldn't create a fence! => {}", e),
            RendererCreationError::ImageAvailableSemaphore(e) => {
                format!("Couldn't create the ImageAvailable semaphore! => {}", e)
            }
            RendererCreationError::RenderFinishedSemaphore(e) => {
                format!("Couldn't create the RenderFinished semaphore! => {}", e)
            }
            RendererCreationError::RenderPassCreation(e) => {
                format!("Couldn't create the RenderPass! => {}", e)
            }
            RendererCreationError::ImageViews(e) => {
                format!("Couldn't create the image view for the image! => {}", e)
            }
            RendererCreationError::FrameBuffers(e) => format!("Couldn't create the framebuffer! => {}", e),
            RendererCreationError::CommandPool(e) => {
                format!("Couldn't create the raw command pool! => {}", e)
            }
        };
        write!(f, "{}", write)
    }
}

#[derive(Debug, Fail)]
pub enum PipelineCreationError {
    ShaderCompilerFailed,
    VertexShaderCompilation(#[cause] shaderc::Error),
    FragmentShaderCompilation(#[cause] shaderc::Error),
    VertexModule(#[cause] ShaderError),
    FragmentModule(#[cause] ShaderError),
    DescriptorSetLayout(#[cause] OutOfMemory),
    DescriptorPool(#[cause] OutOfMemory),
    PipelineLayout(#[cause] OutOfMemory),
    PipelineCreation(#[cause] gfx_hal::pso::CreationError, &'static str),
}

impl std::fmt::Display for PipelineCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write: String = match self {
            PipelineCreationError::ShaderCompilerFailed => format!("Shader compiler was not created!"),
            PipelineCreationError::VertexShaderCompilation(e) => {
                format!("Vertex shader did not compile! => {}", e)
            }
            PipelineCreationError::FragmentShaderCompilation(e) => {
                format!("Fragment shader did not compile! => {}", e)
            }
            PipelineCreationError::VertexModule(e) => format!("Vertex module did not compile! => {}", e),
            PipelineCreationError::FragmentModule(e) => format!("Fragment module did not compile! => {}", e),
            PipelineCreationError::DescriptorSetLayout(e) => {
                format!("DescriptorSetLayout could not be allocated! => {}", e)
            }
            PipelineCreationError::DescriptorPool(e) => {
                format!("DescriptorPool could not be allocated! => {}", e)
            }
            PipelineCreationError::PipelineLayout(e) => {
                format!("PipelineLayout could not be allocated! => {}", e)
            }
            PipelineCreationError::PipelineCreation(e, k) => {
                format!("Pipeline {} could not be created! => {}", k, e)
            }
        };
        write!(f, "{}", write)
    }
}

use gfx_hal::buffer::CreationError;
#[derive(Debug, Fail)]
pub enum BufferBundleError {
    Creation(#[cause] CreationError),
}

impl std::fmt::Display for BufferBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write_this = match self {
            BufferBundleError::Creation(e) => format!("Buffer creation error! => {}", e),
        };

        write!(f, "{}", write_this)
    }
}

#[derive(Debug, Fail)]
pub enum BufferError {
    MemoryId,
    Allocate(#[cause] gfx_hal::device::AllocationError),
    Bind(#[cause] gfx_hal::device::BindError),
    Map(#[cause] gfx_hal::mapping::Error),
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write_this = match self {
            BufferError::MemoryId => format!("MemoryID Error"),
            BufferError::Allocate(e) => format!("Buffer allocation error! => {}", e),
            BufferError::Bind(e) => format!("Buffer binding error! => {}", e),
            BufferError::Map(e) => format!("Buffer mapping error! => {}", e),
        };

        write!(f, "{}", write_this)
    }
}

#[derive(Debug, Fail)]
pub enum LoadedImageError {
    AcquireMappingWriter(#[cause] gfx_hal::mapping::Error),
    ReleaseMappingWriter(#[cause] OutOfMemory),
    CreateImage(#[cause] gfx_hal::image::CreationError),
    ImageView(#[cause] gfx_hal::image::ViewError),
    Sampler(#[cause] gfx_hal::device::AllocationError),
    UploadFence(#[cause] OutOfMemory),
    WaitForFence(#[cause] OomOrDeviceLost),
}

impl std::fmt::Display for LoadedImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write_this = match self {
            LoadedImageError::AcquireMappingWriter(e) => format!(
                "Couldn't acquire a mapping writer to the staging buffer! => {}",
                e
            ),
            LoadedImageError::ReleaseMappingWriter(e) => format!(
                "Couldn't release the mapping writer to the staging buffer! => {}",
                e
            ),
            LoadedImageError::CreateImage(e) => format!("Couldn't create the image! => {}", e),
            LoadedImageError::ImageView(e) => format!("Couldn't create the image view! => {}", e),
            LoadedImageError::Sampler(e) => format!("Couldn't create the sampler! => {}", e),
            LoadedImageError::UploadFence(e) => format!("Couldn't create the upload fence! => {}", e),
            LoadedImageError::WaitForFence(e) => format!("Couldn't wait for the fence! => {}", e),
        };

        write!(f, "{}", write_this)
    }
}

#[derive(Debug, Fail)]
pub enum MemoryWritingError {
    #[fail(
        display = "Couldn't acquire a mapping writer to the staging buffer! => {}",
        _0
    )]
    AcquireMappingWriter(#[cause] gfx_hal::mapping::Error),
    #[fail(
        display = "Couldn't release the mapping writer to the staging buffer! => {}",
        _0
    )]
    ReleaseMappingWriter(#[cause] OutOfMemory),
}

quick_from!(
    MemoryWritingError,
    MemoryWritingError::AcquireMappingWriter,
    gfx_hal::mapping::Error
);

quick_from!(
    MemoryWritingError,
    MemoryWritingError::ReleaseMappingWriter,
    OutOfMemory
);
