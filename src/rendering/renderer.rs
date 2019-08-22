use arrayvec::ArrayVec;
use core::mem::ManuallyDrop;
use gfx_hal::{
    adapter::{Adapter, PhysicalDevice},
    buffer::{self, IndexBufferView},
    command::{ClearColor, ClearValue, CommandBuffer, MultiShot, Primary},
    device::Device,
    format::{Aspects, ChannelType, Format, Swizzle},
    image::{Extent, Layout, SubresourceRange, Usage, ViewKind},
    pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, Subpass, SubpassDesc},
    pool::{CommandPool, CommandPoolCreateFlags},
    pso::{
        BakedStates, BasePipeline, BlendDesc, BlendOp, BlendState, ColorBlendDesc, ColorMask,
        DepthStencilDesc, DescriptorSetLayoutBinding, ElemStride, EntryPoint, Face, Factor, FrontFace,
        GraphicsPipelineDesc, GraphicsShaderSet, InputAssemblerDesc, LogicOp, PipelineCreationFlags,
        PipelineStage, PolygonMode, Rasterizer, Rect, ShaderStageFlags, Specialization, VertexBufferDesc,
        VertexInputRate, Viewport,
    },
    queue::{family::QueueGroup, Submission},
    window::{Extent2D, PresentMode, Suboptimal, Surface, Swapchain, SwapchainConfig},
    Backend, Features, Gpu, Graphics, IndexType, Instance, Primitive, QueueFamily,
};
use lokacore;
use nalgebra_glm as glm;
use std::{borrow::Cow, mem, ops::Deref};
use winit::Window as WinitWindow;

#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use super::{BufferBundle, Entity, Vertex, Window, QUAD_INDICES, QUAD_VERTICES};

pub const VERTEX_SOURCE: &str = include_str!("shaders/vert_default.vert");
pub const FRAGMENT_SOURCE: &str = include_str!("shaders/frag_default.frag");

#[allow(dead_code)]
pub struct Renderer<I: Instance> {
    // Top
    instance: ManuallyDrop<I>,
    surface: <I::Backend as Backend>::Surface,
    adapter: Adapter<I::Backend>,
    queue_group: ManuallyDrop<QueueGroup<I::Backend, Graphics>>,
    device: ManuallyDrop<<I::Backend as Backend>::Device>,

    format: Format,

    // Pipeline nonsense
    vertices: BufferBundle<I::Backend>,
    indexes: BufferBundle<I::Backend>,
    descriptor_set_layouts: Vec<<I::Backend as Backend>::DescriptorSetLayout>,
    pipeline_layout: ManuallyDrop<<I::Backend as Backend>::PipelineLayout>,
    graphics_pipeline: ManuallyDrop<<I::Backend as Backend>::GraphicsPipeline>,

    // GPU Swapchain
    swapchain: ManuallyDrop<<I::Backend as Backend>::Swapchain>,
    viewport: Rect,
    in_flight_fences: Vec<<I::Backend as Backend>::Fence>,
    frames_in_flight: usize,
    image_available_semaphores: Vec<<I::Backend as Backend>::Semaphore>,
    render_finished_semaphores: Vec<<I::Backend as Backend>::Semaphore>,

    // Render Pass
    render_pass: ManuallyDrop<<I::Backend as Backend>::RenderPass>,

    // Render Targets
    image_views: Vec<(<I::Backend as Backend>::ImageView)>,
    framebuffers: Vec<<I::Backend as Backend>::Framebuffer>,

    // Command Issues
    command_pool: ManuallyDrop<CommandPool<I::Backend, Graphics>>,
    command_buffers: Vec<CommandBuffer<I::Backend, Graphics, MultiShot, Primary>>,

    // Mis
    current_frame: usize,
}

pub type TypedRenderer = Renderer<back::Instance>;
impl<I: Instance> Renderer<I> {
    pub fn typed_new(window: &Window) -> Result<TypedRenderer, &'static str> {
        // Create An Instance
        let instance = back::Instance::create(window.name, 1);
        // Create A Surface
        let surface = instance.create_surface(&window.window);
        // Create A renderer
        Ok(TypedRenderer::new(&window.window, instance, surface)?)
    }

    pub fn new(
        window: &WinitWindow,
        instance: I,
        mut surface: <I::Backend as Backend>::Surface,
    ) -> Result<Self, &'static str> {
        let adapter = instance
            .enumerate_adapters()
            .into_iter()
            .find(|a| {
                a.queue_families
                    .iter()
                    .any(|qf| qf.supports_graphics() && surface.supports_queue_family(qf))
            })
            .ok_or("Couldn't find a graphical adapter!")?;

        // open it up!
        let (mut device, queue_group) = {
            let queue_family = adapter
                .queue_families
                .iter()
                .find(|qf| qf.supports_graphics() && surface.supports_queue_family(qf))
                .ok_or("Couldn't find a QueueFamily with graphics!")?;

            let Gpu { device, mut queues } = unsafe {
                adapter
                    .physical_device
                    .open(&[(queue_family, &[1.0; 1])], Features::empty())
                    .map_err(|_| "Couldn't open the PhysicalDevice!")?
            };

            let queue_group = queues
                .take::<Graphics>(queue_family.id())
                .expect("Couldn't take ownership of the QueueGroup!");

            if queue_group.queues.len() == 0 {
                return Err("The QueueGroup did not have any CommandQueues available");
            }
            (device, queue_group)
        };

        let (swapchain, extent, backbuffer, format, frames_in_flight) = {
            // no composite alpha here
            let (caps, preferred_formats, present_modes) = surface.compatibility(&adapter.physical_device);
            trace!("{:?}", caps);
            trace!("Preferred Formats: {:?}", preferred_formats);
            trace!("Present Modes: {:?}", present_modes);

            let present_mode = {
                use gfx_hal::window::PresentMode::*;
                [Mailbox, Fifo, Relaxed, Immediate]
                    .iter()
                    .cloned()
                    .find(|pm| present_modes.contains(pm))
                    .ok_or("No PresentMode values specified!")?
            };

            use gfx_hal::window::CompositeAlpha;
            trace!("We're setting composite alpha to opaque...Need to figure out where to find the user's intent.");
            let composite_alpha = CompositeAlpha::OPAQUE;

            let format = match preferred_formats {
                None => Format::Rgba8Srgb,
                Some(formats) => match formats
                    .iter()
                    .find(|format| format.base_format().1 == ChannelType::Srgb)
                    .cloned()
                {
                    Some(srgb_format) => srgb_format,
                    None => formats
                        .get(0)
                        .cloned()
                        .ok_or("Preferred format list was empty!")?,
                },
            };

            let extent = {
                let window_client_area = window
                    .get_inner_size()
                    .ok_or("Window doesn't exist!")?
                    .to_physical(window.get_hidpi_factor());

                Extent2D {
                    width: caps.extents.end().width.min(window_client_area.width as u32),
                    height: caps.extents.end().height.min(window_client_area.height as u32),
                }
            };

            let image_count = if present_mode == PresentMode::Mailbox {
                (caps.image_count.end() - 1).min(*caps.image_count.start().max(&3))
            } else {
                (caps.image_count.end() - 1).min(*caps.image_count.start().max(&2))
            };

            let image_layers = 1;
            if caps.usage.contains(Usage::COLOR_ATTACHMENT) == false {
                return Err("The Surface isn't capable of supporting color!");
            }

            let image_usage = Usage::COLOR_ATTACHMENT;

            let swapchain_config = SwapchainConfig {
                present_mode,
                composite_alpha,
                format,
                extent,
                image_count,
                image_layers,
                image_usage,
            };

            trace!("{:?}", swapchain_config);

            // Final pop out. PHEW!
            let (swapchain, backbuffer) = unsafe {
                device
                    .create_swapchain(&mut surface, swapchain_config, None)
                    .map_err(|_| "Failed to create the swapchain on the last step!")?
            };

            (swapchain, extent, backbuffer, format, image_count as usize)
        };

        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) = {
            let mut image_available_semaphores = vec![];
            let mut render_finished_semaphores = vec![];
            let mut in_flight_fences = vec![];
            for _ in 0..frames_in_flight {
                in_flight_fences.push(
                    device
                        .create_fence(true)
                        .map_err(|_| "Could not create a fence!")?,
                );
                image_available_semaphores.push(
                    device
                        .create_semaphore()
                        .map_err(|_| "Could not create a semaphore!")?,
                );
                render_finished_semaphores.push(
                    device
                        .create_semaphore()
                        .map_err(|_| "Could not create a semaphore!")?,
                );
            }
            (
                image_available_semaphores,
                render_finished_semaphores,
                in_flight_fences,
            )
        };

        let render_pass = {
            let color_attachment = Attachment {
                format: Some(format),
                samples: 1,
                ops: AttachmentOps {
                    load: AttachmentLoadOp::Clear,
                    store: AttachmentStoreOp::Store,
                },
                stencil_ops: AttachmentOps::DONT_CARE,
                layouts: Layout::Undefined..Layout::Present,
            };

            let subpass = SubpassDesc {
                colors: &[(0, Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            unsafe {
                device
                    .create_render_pass(&[color_attachment], &[subpass], &[])
                    .map_err(|_| "Couldn't create a render pass!")?
            }
        };

        let image_views = {
            backbuffer
                .into_iter()
                .map(|image| unsafe {
                    device
                        .create_image_view(
                            &image,
                            ViewKind::D2,
                            format,
                            Swizzle::NO,
                            SubresourceRange {
                                aspects: Aspects::COLOR,
                                levels: 0..1,
                                layers: 0..1,
                            },
                        )
                        .map_err(|_| "Couldn't create the image_view for the image!")
                })
                .collect::<Result<Vec<_>, &str>>()?
        };

        let framebuffers = {
            image_views
                .iter()
                .map(|image_view| unsafe {
                    device
                        .create_framebuffer(
                            &render_pass,
                            vec![image_view],
                            Extent {
                                width: extent.width as u32,
                                height: extent.height as u32,
                                depth: 1,
                            },
                        )
                        .map_err(|_| "Failed to create a framebuffer!")
                })
                .collect::<Result<Vec<_>, &str>>()?
        };

        let mut command_pool = unsafe {
            device
                .create_command_pool_typed(&queue_group, CommandPoolCreateFlags::RESET_INDIVIDUAL)
                .map_err(|_| "Could not create the raw command pool!")?
        };

        let command_buffers: Vec<_> = framebuffers
            .iter()
            .map(|_| command_pool.acquire_command_buffer())
            .collect();

        let (descriptor_set_layouts, pipeline_layout, graphics_pipeline) =
            Self::create_pipeline(&mut device, extent, &render_pass)?;

        let vertices = BufferBundle::new(
            &adapter,
            &device,
            mem::size_of_val(&QUAD_VERTICES) as u64,
            buffer::Usage::VERTEX,
        )?;
        Renderer::<I>::bind_to_memory(&mut device, &vertices, &QUAD_VERTICES)?;

        let indexes = BufferBundle::new(
            &adapter,
            &device,
            mem::size_of_val(&QUAD_INDICES) as u64,
            buffer::Usage::INDEX,
        )?;
        Renderer::<I>::bind_to_memory(&mut device, &indexes, &QUAD_INDICES)?;

        Ok(Self {
            instance: manual_new!(instance),
            surface,
            adapter,
            format,
            device: manual_new!(device),
            queue_group: manual_new!(queue_group),
            swapchain: manual_new!(swapchain),
            viewport: extent.to_extent().rect(),
            render_pass: manual_new!(render_pass),
            image_views,
            framebuffers,
            command_pool: manual_new!(command_pool),
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            frames_in_flight,
            current_frame: 0,

            vertices,
            indexes,
            descriptor_set_layouts,
            pipeline_layout: manual_new!(pipeline_layout),
            graphics_pipeline: manual_new!(graphics_pipeline),
        })
    }

    fn create_pipeline(
        device: &mut <I::Backend as Backend>::Device,
        extent: Extent2D,
        render_pass: &<I::Backend as Backend>::RenderPass,
    ) -> Result<
        (
            Vec<<I::Backend as Backend>::DescriptorSetLayout>,
            <I::Backend as Backend>::PipelineLayout,
            <I::Backend as Backend>::GraphicsPipeline,
        ),
        &'static str,
    > {
        let mut compiler = shaderc::Compiler::new().ok_or("shaderc not found!")?;
        let vertex_compile_artifact = compiler
            .compile_into_spirv(
                VERTEX_SOURCE,
                shaderc::ShaderKind::Vertex,
                "vertex.vert",
                "main",
                None,
            )
            .map_err(|_| "Couldn't compile vertex shader!")?;

        let fragment_compile_artifact = compiler
            .compile_into_spirv(
                FRAGMENT_SOURCE,
                shaderc::ShaderKind::Fragment,
                "fragment.frag",
                "main",
                None,
            )
            .map_err(|e| {
                error!("{}", e);
                "Couldn't compile fragment shader!"
            })?;

        let vertex_shader_module = unsafe {
            device
                .create_shader_module(vertex_compile_artifact.as_binary())
                .map_err(|_| "Couldn't make the vertex module!")?
        };

        let fragment_shader_module = unsafe {
            device
                .create_shader_module(fragment_compile_artifact.as_binary())
                .map_err(|_| "Couldn't make the fragment module!")?
        };

        let (vs_entry, fs_entry) = (
            EntryPoint {
                entry: "main",
                module: &vertex_shader_module,
                specialization: Specialization {
                    constants: Cow::Borrowed(&[]),
                    data: Cow::Borrowed(&[]),
                },
            },
            EntryPoint {
                entry: "main",
                module: &fragment_shader_module,
                specialization: Specialization {
                    constants: Cow::Borrowed(&[]),
                    data: Cow::Borrowed(&[]),
                },
            },
        );

        let input_assembler = InputAssemblerDesc::new(Primitive::TriangleList);

        let shaders = GraphicsShaderSet {
            vertex: vs_entry,
            fragment: Some(fs_entry),
            domain: None,
            geometry: None,
            hull: None,
        };

        let vertex_buffers = vec![VertexBufferDesc {
            binding: 0,
            stride: mem::size_of::<Vertex>() as ElemStride,
            rate: VertexInputRate::Vertex,
        }];

        let attributes = Vertex::attributes();

        let rasterizer = Rasterizer {
            depth_clamping: false,
            polygon_mode: PolygonMode::Fill,
            cull_face: Face::NONE,
            front_face: FrontFace::Clockwise,
            depth_bias: None,
            conservative: false,
        };

        let depth_stencil = DepthStencilDesc {
            depth: None,
            depth_bounds: false,
            stencil: None,
        };

        let blender = {
            let blend_state = BlendState {
                color: BlendOp::Add {
                    src: Factor::One,
                    dst: Factor::Zero,
                },
                alpha: BlendOp::Add {
                    src: Factor::One,
                    dst: Factor::Zero,
                },
            };
            BlendDesc {
                logic_op: Some(LogicOp::Copy),
                targets: vec![ColorBlendDesc {
                    mask: ColorMask::ALL,
                    blend: Some(blend_state),
                }],
            }
        };

        let baked_states = BakedStates {
            viewport: Some(Viewport {
                rect: extent.to_extent().rect(),
                depth: (0.0..1.0),
            }),
            scissor: Some(extent.to_extent().rect()),
            blend_color: None,
            depth_bounds: None,
        };

        let bindings = Vec::<DescriptorSetLayoutBinding>::new();
        let immutable_sampler: Vec<<I::Backend as Backend>::Sampler> = Vec::new();

        let descriptor_set_layouts = vec![unsafe {
            device
                .create_descriptor_set_layout(bindings, immutable_sampler)
                .map_err(|_| "Couldn't make a Descriptor Set Layout!")?
        }];

        let push_constants = vec![
            (ShaderStageFlags::VERTEX, 0..16),
            (ShaderStageFlags::FRAGMENT, 0..3),
        ];
        let layout = unsafe {
            device
                .create_pipeline_layout(&descriptor_set_layouts, push_constants)
                .map_err(|_| "Couldn't create a pipeline layout")?
        };

        let gfx_pipeline = {
            let desc = GraphicsPipelineDesc {
                shaders,
                rasterizer,
                vertex_buffers,
                attributes,
                input_assembler,
                blender,
                depth_stencil,
                multisampling: None,
                baked_states,
                layout: &layout,
                subpass: Subpass {
                    index: 0,
                    main_pass: render_pass,
                },
                flags: PipelineCreationFlags::empty(),
                parent: BasePipeline::None,
            };

            unsafe {
                device
                    .create_graphics_pipeline(&desc, None)
                    .map_err(|_| "Couldn't create a graphics pipeline!")?
            }
        };

        Ok((descriptor_set_layouts, layout, gfx_pipeline))
    }

    fn bind_to_memory<T: Copy>(
        device: &mut <I::Backend as Backend>::Device,
        buffer_bundle: &BufferBundle<I::Backend>,
        data: &'static [T],
    ) -> Result<(), &'static str> {
        unsafe {
            let mut data_target = device
                .acquire_mapping_writer(&buffer_bundle.memory, 0..buffer_bundle.requirements.size)
                .map_err(|_| "Failed to acquire an buffer mapping writer!")?;

            data_target[..data.len()].copy_from_slice(&data);

            device
                .release_mapping_writer(data_target)
                .map_err(|_| "Couldn't release the buffer mapping writer!")?;
        };

        Ok(())
    }

    pub fn draw_quad_frame(
        &mut self,
        entities: &[Vec<Entity>],
        view_projection: &glm::TMat4<f32>,
        aspect_ratio: &f32,
        scale: &f32,
    ) -> Result<Option<Suboptimal>, DrawingError> {
        // SETUP FOR THIS FRAME
        let image_available = &self.image_available_semaphores[self.current_frame];
        let render_finished = &self.render_finished_semaphores[self.current_frame];
        // Advance the frame _before_ we start using the `?` operator
        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;

        let (i_u32, i_usize) = unsafe {
            let image_index = self
                .swapchain
                .acquire_image(core::u64::MAX, Some(image_available), None)
                .map_err(|_| DrawingError::AcquireAnImageFromSwapchain)?;

            (image_index.0, image_index.0 as usize)
        };

        // Get the fence, and wait for the fence
        let flight_fence = &self.in_flight_fences[i_usize];
        unsafe {
            self.device
                .wait_for_fence(flight_fence, core::u64::MAX)
                .map_err(|_| DrawingError::WaitOnFence)?;
            self.device
                .reset_fence(flight_fence)
                .map_err(|_| DrawingError::ResetFence)?;
        }

        // RECORD COMMANDS
        unsafe {
            let buffer = &mut self.command_buffers[i_usize];
            const TRIANGLE_CLEAR: [ClearValue; 1] = [ClearValue::Color(ClearColor::Sfloat([
                139.0 / 255.0,
                110.0 / 255.0,
                101.0 / 255.0,
                1.0,
            ]))];
            buffer.begin(false);
            {
                let mut encoder = buffer.begin_render_pass_inline(
                    &self.render_pass,
                    &self.framebuffers[i_usize],
                    self.viewport,
                    TRIANGLE_CLEAR.iter(),
                );
                encoder.bind_graphics_pipeline(&self.graphics_pipeline);

                // Bind the vertex buffers in
                encoder.bind_vertex_buffers(0, Some((self.vertices.buffer.deref(), 0)));
                encoder.bind_index_buffer(IndexBufferView {
                    buffer: &self.indexes.buffer,
                    offset: 0,
                    index_type: IndexType::U16,
                });

                for row in entities {
                    for entity in row {
                        let mvp = {
                            let position_matrix = glm::scale(
                                &view_projection,
                                &glm::make_vec3(&[*scale, scale * aspect_ratio, 1.0]),
                            );
                            position_matrix * entity.coordinate.into_vec2().into_glm_tmat4(0.0)
                        };

                        encoder.push_graphics_constants(
                            &self.pipeline_layout,
                            ShaderStageFlags::VERTEX,
                            0,
                            lokacore::cast_slice::<f32, u32>(&mvp.data),
                        );
                        encoder.push_graphics_constants(
                            &self.pipeline_layout,
                            ShaderStageFlags::FRAGMENT,
                            0,
                            &entity.state.to_color_bits(),
                        );
                        encoder.draw_indexed(0..6, 0, 0..1);
                    }
                }
            }
            buffer.finish();
        }

        // SUBMISSION AND PRESENT
        let command_buffers = &self.command_buffers[i_usize..=i_usize];
        let wait_semaphores: ArrayVec<[_; 1]> =
            [(image_available, PipelineStage::COLOR_ATTACHMENT_OUTPUT)].into();
        let signal_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
        // yes, you have to write it twice like this. yes, it's silly.
        let present_wait_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
        let submission = Submission {
            command_buffers,
            wait_semaphores,
            signal_semaphores,
        };
        let the_command_queue = &mut self.queue_group.queues[0];
        unsafe {
            the_command_queue.submit(submission, Some(flight_fence));
            self.swapchain
                .present(the_command_queue, i_u32, present_wait_semaphores)
                .map_err(|_| DrawingError::PresentIntoSwapchain)
        }
    }

    pub fn recreate_swapchain(&mut self, window: &WinitWindow) -> Result<(), &'static str> {
        let (caps, formats, _) = self.surface.compatibility(&mut self.adapter.physical_device);
        assert!(formats.iter().any(|fs| fs.contains(&self.format)));

        let extent = {
            let window_client_area = window
                .get_inner_size()
                .ok_or("Window doesn't exist!")?
                .to_physical(window.get_hidpi_factor());

            Extent2D {
                width: caps.extents.end().width.min(window_client_area.width as u32),
                height: caps.extents.end().height.min(window_client_area.height as u32),
            }
        };

        self.viewport = extent.to_extent().rect();

        let swapchain_config = gfx_hal::window::SwapchainConfig::from_caps(&caps, self.format, extent);

        unsafe {
            let (swapchain, backbuffer) = self
                .device
                .create_swapchain(&mut self.surface, swapchain_config, None)
                .map_err(|_| "Couldn't recreate the swapchain!")?;

            self.drop_swapchain()?;

            let image_views = {
                backbuffer
                    .into_iter()
                    .map(|image| {
                        self.device
                            .create_image_view(
                                &image,
                                ViewKind::D2,
                                self.format,
                                Swizzle::NO,
                                SubresourceRange {
                                    aspects: Aspects::COLOR,
                                    levels: 0..1,
                                    layers: 0..1,
                                },
                            )
                            .map_err(|_| "Couldn't create the image_view for the image!")
                    })
                    .collect::<Result<Vec<_>, &str>>()?
            };

            let framebuffers = {
                image_views
                    .iter()
                    .map(|image_view| {
                        self.device
                            .create_framebuffer(
                                &self.render_pass,
                                vec![image_view],
                                Extent {
                                    width: extent.width as u32,
                                    height: extent.height as u32,
                                    depth: 1,
                                },
                            )
                            .map_err(|_| "Failed to create a framebuffer!")
                    })
                    .collect::<Result<Vec<_>, &str>>()?
            };

            let mut command_pool = self
                .device
                .create_command_pool_typed(&self.queue_group, CommandPoolCreateFlags::RESET_INDIVIDUAL)
                .map_err(|_| "Could not create the raw command pool!")?;

            let command_buffers: Vec<CommandBuffer<I::Backend, Graphics, MultiShot, Primary>> = framebuffers
                .iter()
                .map(|_| command_pool.acquire_command_buffer())
                .collect();

            let (descriptor_set_layouts, pipeline_layout, graphics_pipeline) =
                Self::create_pipeline(&mut self.device, extent, &self.render_pass)?;

            self.descriptor_set_layouts = descriptor_set_layouts;
            self.pipeline_layout = manual_new!(pipeline_layout);
            self.graphics_pipeline = manual_new!(graphics_pipeline);

            // Finally, we got ourselves a nice and shiny new swapchain!
            self.swapchain = manual_new!(swapchain);
            self.framebuffers = framebuffers;
            self.command_buffers = command_buffers;
            self.command_pool = manual_new!(command_pool);
        }
        Ok(())
    }

    fn drop_swapchain(&mut self) -> Result<(), &'static str> {
        self.device.wait_idle().unwrap();

        use core::ptr::read;
        unsafe {
            for framebuffer in self.framebuffers.drain(..) {
                self.device.destroy_framebuffer(framebuffer);
            }
            self.device
                .destroy_command_pool(manual_drop!(self.command_pool).into_raw());

            for this_layout in self.descriptor_set_layouts.drain(..) {
                self.device.destroy_descriptor_set_layout(this_layout);
            }
            self.device
                .destroy_pipeline_layout(manual_drop!(self.pipeline_layout));
            self.device
                .destroy_graphics_pipeline(manual_drop!(self.graphics_pipeline));

            self.device.destroy_swapchain(manual_drop!(self.swapchain));
        }

        Ok(())
    }
}

impl<I: Instance> core::ops::Drop for Renderer<I> {
    fn drop(&mut self) {
        self.device.wait_idle().unwrap();

        unsafe {
            for fence in self.in_flight_fences.drain(..) {
                self.device.destroy_fence(fence);
            }

            for semaphore in self.render_finished_semaphores.drain(..) {
                self.device.destroy_semaphore(semaphore)
            }
            for semaphore in self.image_available_semaphores.drain(..) {
                self.device.destroy_semaphore(semaphore)
            }
            for framebuffer in self.framebuffers.drain(..) {
                self.device.destroy_framebuffer(framebuffer);
            }
            for image_view in self.image_views.drain(..) {
                self.device.destroy_image_view(image_view);
            }

            for this_layout in self.descriptor_set_layouts.drain(..) {
                self.device.destroy_descriptor_set_layout(this_layout);
            }

            // LAST RESORT STYLE CODE, NOT TO BE IMITATED LIGHTLY
            use core::ptr::read;
            self.vertices.manually_drop(&self.device);
            self.indexes.manually_drop(&self.device);

            self.device
                .destroy_pipeline_layout(manual_drop!(self.pipeline_layout));
            self.device
                .destroy_graphics_pipeline(manual_drop!(self.graphics_pipeline));
            self.device
                .destroy_command_pool(manual_drop!(self.command_pool).into_raw());
            self.device.destroy_render_pass(manual_drop!(self.render_pass));
            self.device.destroy_swapchain(manual_drop!(self.swapchain));

            ManuallyDrop::drop(&mut self.device);
            ManuallyDrop::drop(&mut self.instance);
        }
    }
}

#[derive(Debug)]
pub enum DrawingError {
    AcquireAnImageFromSwapchain,
    WaitOnFence,
    ResetFence,
    PresentIntoSwapchain,
}
