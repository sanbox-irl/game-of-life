use core::mem::ManuallyDrop;
use gfx_hal::{device::Device, Backend, DescriptorPool};

pub struct PipelineBundle<B: Backend> {
    pub descriptor_set_layout: Option<<B as Backend>::DescriptorSetLayout>,
    pub descriptor_pool: Option<B::DescriptorPool>,
    pub pipeline_layout: ManuallyDrop<<B as Backend>::PipelineLayout>,
    pub graphics_pipeline: ManuallyDrop<<B as Backend>::GraphicsPipeline>,
}

impl<B: Backend> PipelineBundle<B> {
    pub fn new(
        descriptor_set_layout: B::DescriptorSetLayout,
        descriptor_pool: Option<B::DescriptorPool>,
        pipeline_layout: B::PipelineLayout,
        graphics_pipeline: B::GraphicsPipeline,
    ) -> Self {
        PipelineBundle {
            descriptor_set_layout: Some(descriptor_set_layout),
            pipeline_layout: manual_new!(pipeline_layout),
            descriptor_pool,
            graphics_pipeline: manual_new!(graphics_pipeline),
        }
    }

    pub fn allocate_descriptor_set(&mut self) -> Result<B::DescriptorSet, &'static str> {
        match &mut self.descriptor_pool {
            Some(dp) => unsafe {
                if let Some(descriptor_set_layout) = &self.descriptor_set_layout {
                    dp.allocate_set(descriptor_set_layout)
                        .map_err(|_| "Couldn't allocate a descriptor set!")
                } else {
                    Err("Couldn't find the descriptor layout!")
                }
            },

            None => Err(
                "No descriptor pool has been created, but attempting to allocate a descriptor set!
                Please make a descriptor pool first!",
            ),
        }
    }

    pub unsafe fn manually_drop(self, device: &B::Device) {
        use core::ptr::read;
        if let Some(this_layout) = self.descriptor_set_layout {
            device.destroy_descriptor_set_layout(this_layout);
        }
        device.destroy_pipeline_layout(manual_drop!(self.pipeline_layout));
        device.destroy_graphics_pipeline(manual_drop!(self.graphics_pipeline));
    }
}
