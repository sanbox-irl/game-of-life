use core::mem::ManuallyDrop;
use gfx_hal::{
    device::Device,
    Backend,
};

pub struct PipelineBundle<B: Backend> {
    pub descriptor_set_layouts: Vec<<B as Backend>::DescriptorSetLayout>,
    pub pipeline_layout: ManuallyDrop<<B as Backend>::PipelineLayout>,
    pub graphics_pipeline: ManuallyDrop<<B as Backend>::GraphicsPipeline>,
}

impl<B: Backend> PipelineBundle<B> {
    pub unsafe fn manually_drop(&mut self, device: &B::Device) {
        use core::ptr::read;
        for this_layout in self.descriptor_set_layouts.drain(..) {
            device.destroy_descriptor_set_layout(this_layout);
        }
        device.destroy_pipeline_layout(manual_drop!(self.pipeline_layout));
        device.destroy_graphics_pipeline(manual_drop!(self.graphics_pipeline));
    }
}
