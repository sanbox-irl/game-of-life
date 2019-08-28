use core::mem::ManuallyDrop;
use gfx_hal::{device::Device, Backend};

pub struct PipelineBundle<B: Backend> {
    pub descriptor_set_layouts: Option<<B as Backend>::DescriptorSetLayout>,
    pub pipeline_layout: ManuallyDrop<<B as Backend>::PipelineLayout>,
    pub graphics_pipeline: ManuallyDrop<<B as Backend>::GraphicsPipeline>,
}

impl<B: Backend> PipelineBundle<B> {
    pub fn new(
        descriptor_set_layout: B::DescriptorSetLayout,
        pipeline_layout: B::PipelineLayout,
        graphics_pipeline: B::GraphicsPipeline,
    ) -> Self {
        PipelineBundle {
            descriptor_set_layouts: Some(descriptor_set_layout),
            pipeline_layout: manual_new!(pipeline_layout),
            graphics_pipeline: manual_new!(graphics_pipeline),
        }
    }

    pub unsafe fn manually_drop(self, device: &B::Device) {
        use core::ptr::read;
        if let Some(this_layout) = self.descriptor_set_layouts {
                device.destroy_descriptor_set_layout(this_layout);
        }
        device.destroy_pipeline_layout(manual_drop!(self.pipeline_layout));
        device.destroy_graphics_pipeline(manual_drop!(self.graphics_pipeline));
    }
}
