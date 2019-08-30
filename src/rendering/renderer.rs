use arrayvec::ArrayVec;
use core::mem::ManuallyDrop;
use gfx_hal::{
    adapter::{Adapter, PhysicalDevice},
    buffer::{self, IndexBufferView},
    command::{ClearColor, ClearValue, CommandBuffer, MultiShot, Primary, RenderPassInlineEncoder},
    device::Device,
    format::{Aspects, ChannelType, Format, Swizzle},
    image::{Extent, Layout, SubresourceRange, Usage, ViewKind},
    pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, Subpass, SubpassDesc},
    pool::{CommandPool, CommandPoolCreateFlags},
    pso::{
        AttributeDesc, BakedStates, BasePipeline, BlendDesc, BlendOp, BlendState, ColorBlendDesc, ColorMask,
        DepthStencilDesc, DescriptorRangeDesc, DescriptorSetLayoutBinding, DescriptorType, ElemStride,
        Element, EntryPoint, Face, Factor, FrontFace, GraphicsPipelineDesc, GraphicsShaderSet,
        InputAssemblerDesc, LogicOp, PipelineCreationFlags, PipelineStage, PolygonMode, Rasterizer, Rect,
        ShaderStageFlags, Specialization, VertexBufferDesc, VertexInputRate, Viewport,
    },
    queue::{family::QueueGroup, Submission},
    window::{Extent2D, PresentMode, Suboptimal, Surface, Swapchain, SwapchainConfig},
    Backend, Features, Gpu, Graphics, IndexType, Instance, Primitive, QueueFamily,
};
use imgui::{Context as ImGuiContext, DrawIdx, DrawVert, TextureId};
use std::{borrow::Cow, mem, ops::Deref};
use winit::Window as WinitWindow;

#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use super::{
    BufferBundle, BufferBundleError, DrawingError, GameWorldDrawCommands, ImGuiDrawCommands, LoadedImage,
    PipelineBundle, RendererCommands, Vertex, VertexIndexPairBufferBundle, Window, QUAD_INDICES,
    QUAD_VERTICES,
};

const VERTEX_PUSH_CONSTANTS_SIZE: u32 = 6;
const FRAG_PUSH_CONSTANTS_START: u32 = 8;
const FRAG_PUSH_CONSTANTS_SIZE: u32 = 3;

const QUAD_DATA: usize = 0;
const IMGUI_DATA: usize = 1;
const PIPELINE_SIZE: usize = 2;

pub struct Renderer<I: Instance> {
    // Top
    instance: ManuallyDrop<I>,
    surface: <I::Backend as Backend>::Surface,
    adapter: Adapter<I::Backend>,
    queue_group: ManuallyDrop<QueueGroup<I::Backend, Graphics>>,
    device: ManuallyDrop<<I::Backend as Backend>::Device>,
    format: Format,

    // Pipeline nonsense
    pipeline_bundles: ArrayVec<[PipelineBundle<I::Backend>; PIPELINE_SIZE]>,
    vertex_index_buffer_bundles: ArrayVec<[VertexIndexPairBufferBundle<I::Backend>; PIPELINE_SIZE]>,
    images: ArrayVec<[LoadedImage<I::Backend>; 1]>,

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
    pub fn typed_new(window: &Window, imgui: &mut ImGuiContext) -> Result<TypedRenderer, &'static str> {
        // Create An Instance
        let instance = back::Instance::create(window.name, 1);
        // Create A Surface
        let surface = instance.create_surface(&window.window);
        // Create A renderer
        let mut renderer = TypedRenderer::new(&window.window, instance, surface)?;
        // Allocate our Textures -- spin this out to another method if we ever make another texture
        renderer.allocate_imgui_textures(imgui)?;
        renderer
            .allocate_imgui_buffers()
            .map_err(|_| "Error allocating the ImGui Buffers!")?;
        Ok(renderer)
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

        // CREATE PIPELINES
        let mut pipeline_bundles = ArrayVec::new();
        pipeline_bundles.push(Self::create_pipeline(&mut device, &extent, &render_pass)?);
        pipeline_bundles.push(Self::create_imgui_pipeline(&device, &render_pass)?);

        // CREATE VERT-INDEX BUFFERS
        let mut vertex_index_buffer_bundles = ArrayVec::new();
        let vertex_buffer = BufferBundle::new(
            &adapter,
            &device,
            mem::size_of_val(&QUAD_VERTICES) as u64,
            buffer::Usage::VERTEX,
        )
        .map_err(|_| "Error Allocating iconic Quad vert buffer!")?;
        Renderer::<I>::bind_to_memory(&mut device, &vertex_buffer, &QUAD_VERTICES)?;

        let index_buffer = BufferBundle::new(
            &adapter,
            &device,
            mem::size_of_val(&QUAD_INDICES) as u64,
            buffer::Usage::INDEX,
        )
        .map_err(|_| "Error Allocating iconic Quad idx buffer!")?;
        Renderer::<I>::bind_to_memory(&mut device, &index_buffer, &QUAD_INDICES)?;

        vertex_index_buffer_bundles.push(VertexIndexPairBufferBundle {
            vertex_buffer,
            index_buffer,
        });

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
            vertex_index_buffer_bundles,

            pipeline_bundles,
            images: ArrayVec::new(),
        })
    }

    fn create_pipeline(
        device: &mut <I::Backend as Backend>::Device,
        extent: &Extent2D,
        render_pass: &<I::Backend as Backend>::RenderPass,
    ) -> Result<(PipelineBundle<I::Backend>), &'static str> {
        const VERTEX_SOURCE: &'static str = include_str!("shaders/default_vert.vert");
        const FRAGMENT_SOURCE: &'static str = include_str!("shaders/default_frag.frag");

        let mut compiler = shaderc::Compiler::new().ok_or("shaderc not found!")?;
        let vertex_compile_artifact = compiler
            .compile_into_spirv(
                VERTEX_SOURCE,
                shaderc::ShaderKind::Vertex,
                "vertex.vert",
                "main",
                None,
            )
            .map_err(|e| {
                error!("{}", e);
                "Couldn't compile vertex shader!"
            })?;

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

        let descriptor_set_layout = Some(unsafe {
            device
                .create_descriptor_set_layout(bindings, immutable_sampler)
                .map_err(|_| "Couldn't make a Descriptor Set Layout!")?
        });

        let push_constants = vec![
            (ShaderStageFlags::VERTEX, 0..VERTEX_PUSH_CONSTANTS_SIZE),
            (
                ShaderStageFlags::FRAGMENT,
                FRAG_PUSH_CONSTANTS_START..FRAG_PUSH_CONSTANTS_START + FRAG_PUSH_CONSTANTS_SIZE,
            ),
        ];
        let layout = unsafe {
            device
                .create_pipeline_layout(&descriptor_set_layout, push_constants)
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

        Ok(PipelineBundle {
            descriptor_set_layout,
            descriptor_pool: None,
            pipeline_layout: manual_new!(layout),
            graphics_pipeline: manual_new!(gfx_pipeline),
        })
    }

    fn create_imgui_pipeline(
        device: &<I::Backend as Backend>::Device,
        render_pass: &<I::Backend as Backend>::RenderPass,
    ) -> Result<PipelineBundle<I::Backend>, &'static str> {
        const IMGUI_VERT_SOURCE: &'static str = include_str!("shaders/imgui_vert.vert");
        const IMGUI_FRAG_SOURCE: &'static str = include_str!("shaders/imgui_frag.frag");

        let mut compiler = shaderc::Compiler::new().ok_or("shaderc not found!")?;
        let vertex_compile_artifact = compiler
            .compile_into_spirv(
                IMGUI_VERT_SOURCE,
                shaderc::ShaderKind::Vertex,
                "imgui_vertex.vert",
                "main",
                None,
            )
            .map_err(|e| {
                error!("{}", e);
                "Couldn't compile vertex shader!"
            })?;

        let fragment_compile_artifact = compiler
            .compile_into_spirv(
                IMGUI_FRAG_SOURCE,
                shaderc::ShaderKind::Fragment,
                "imgui_fragment.frag",
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
                specialization: Specialization::default(),
            },
            EntryPoint {
                entry: "main",
                module: &fragment_shader_module,
                specialization: Specialization::default(),
            },
        );

        let shaders = GraphicsShaderSet {
            vertex: vs_entry,
            fragment: Some(fs_entry),
            domain: None,
            geometry: None,
            hull: None,
        };

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(
                    &[
                        DescriptorSetLayoutBinding {
                            binding: 0,
                            ty: DescriptorType::SampledImage,
                            count: 1,
                            stage_flags: ShaderStageFlags::FRAGMENT,
                            immutable_samplers: false,
                        },
                        DescriptorSetLayoutBinding {
                            binding: 1,
                            ty: DescriptorType::Sampler,
                            count: 1,
                            stage_flags: ShaderStageFlags::FRAGMENT,
                            immutable_samplers: false,
                        },
                    ],
                    &[],
                )
                .map_err(|_| "Couldn't make a DescriptorSetLayout!")?
        };

        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(
                    1,
                    &[
                        DescriptorRangeDesc {
                            ty: DescriptorType::SampledImage,
                            count: 1,
                        },
                        DescriptorRangeDesc {
                            ty: DescriptorType::Sampler,
                            count: 1,
                        },
                    ],
                    gfx_hal::pso::DescriptorPoolCreateFlags::empty(),
                )
                .map_err(|_| "Couldn't create a descriptor pool!")?
        };

        let push_constants = vec![(ShaderStageFlags::VERTEX, 0..4)];
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(Some(&descriptor_set_layout), push_constants)
                .map_err(|_| "Couldn't create the DearImGui pipeline layout")?
        };

        let imgui_pipeline = {
            let mut desc = GraphicsPipelineDesc::new(
                shaders,
                Primitive::TriangleList,
                Rasterizer::FILL,
                &pipeline_layout,
                Subpass {
                    index: 0,
                    main_pass: render_pass,
                },
            );

            desc.vertex_buffers.push(VertexBufferDesc {
                binding: 0,
                stride: mem::size_of::<DrawVert>() as ElemStride,
                rate: VertexInputRate::Vertex,
            });

            desc.attributes.push(
                // Position
                AttributeDesc {
                    location: 0,
                    binding: 0,
                    element: Element {
                        format: Format::Rg32Sfloat,
                        offset: offset_of!(DrawVert, pos) as u32,
                    },
                },
            );

            desc.attributes.push(
                // UV
                AttributeDesc {
                    location: 1,
                    binding: 0,
                    element: Element {
                        format: Format::Rg32Sfloat,
                        offset: offset_of!(DrawVert, uv) as u32,
                    },
                },
            );

            desc.attributes.push(
                // Color
                AttributeDesc {
                    location: 2,
                    binding: 0,
                    element: Element {
                        format: Format::Rgba8Unorm,
                        offset: offset_of!(DrawVert, col) as u32,
                    },
                },
            );

            desc.blender.targets.push(ColorBlendDesc {
                mask: ColorMask::ALL,
                blend: Some(BlendState::ALPHA),
            });

            unsafe {
                device
                    .create_graphics_pipeline(&desc, None)
                    .map_err(|_| "Couldn't create the DearImGui graphics pipeline!")?
            }
        };

        let pipeline_bundle = PipelineBundle::new(
            descriptor_set_layout,
            Some(descriptor_pool),
            pipeline_layout,
            imgui_pipeline,
        );

        Ok(pipeline_bundle)
    }

    fn allocate_imgui_textures(&mut self, imgui: &mut ImGuiContext) -> Result<(), &'static str> {
        let mut fonts = imgui.fonts();
        let imgui::FontAtlasTexture {
            width: font_width,
            height: font_height,
            data: font_data,
        } = fonts.build_rgba32_texture();

        let imgui_image = LoadedImage::allocate_and_create(
            &self.adapter,
            &self.device,
            &mut self.command_pool,
            &mut self.queue_group.queues[0],
            &mut self.pipeline_bundles[IMGUI_DATA],
            font_data,
            font_width as usize,
            font_height as usize,
        )?;

        fonts.tex_id = TextureId::from(self.images.len());

        self.images.push(imgui_image);
        Ok(())
    }

    fn allocate_imgui_buffers(&mut self) -> Result<(), BufferBundleError> {
        let vertex_buffer = BufferBundle::new(
            &self.adapter,
            &self.device,
            (1000 * mem::size_of::<DrawVert>()) as u64,
            buffer::Usage::VERTEX,
        )?;

        let index_buffer = BufferBundle::new(
            &self.adapter,
            &self.device,
            (1000 * mem::size_of::<DrawIdx>()) as u64,
            buffer::Usage::INDEX,
        )?;

        self.vertex_index_buffer_bundles
            .push(VertexIndexPairBufferBundle {
                vertex_buffer,
                index_buffer,
            });

        Ok(())
    }

    pub fn render<'a>(
        &mut self,
        renderer_commands: RendererCommands<'a>,
    ) -> Result<Option<Suboptimal>, DrawingError> {
        // SETUP FOR THIS FRAME
        let image_available = &self.image_available_semaphores[self.current_frame];
        let render_finished = &self.render_finished_semaphores[self.current_frame];
        // Advance the frame *before* we start using the `?` operator
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

                if let Some(game_world_commands) = renderer_commands.game_world_draw_commands {
                    Self::draw_game_world(
                        &mut encoder,
                        game_world_commands,
                        &self.pipeline_bundles[QUAD_DATA],
                        &self.vertex_index_buffer_bundles[QUAD_DATA],
                    )?;
                }

                if let Some(imgui_commands) = renderer_commands.imgui_draw_commands {
                    Self::draw_imgui(
                        &mut encoder,
                        imgui_commands,
                        &self.pipeline_bundles[IMGUI_DATA],
                        &mut self.vertex_index_buffer_bundles[IMGUI_DATA],
                        &self.device,
                        &self.adapter,
                        self.images[0].descriptor_set.deref(),
                    )?;
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

    unsafe fn draw_game_world<'a>(
        encoder: &mut RenderPassInlineEncoder<'_, I::Backend>,
        game_world: GameWorldDrawCommands<'_>,
        quad_pipeline: &'a PipelineBundle<I::Backend>,
        buffer_bundle: &'a VertexIndexPairBufferBundle<I::Backend>,
    ) -> Result<(), DrawingError> {
        encoder.bind_graphics_pipeline(&quad_pipeline.graphics_pipeline);
        // Bind the vertex buffers in
        encoder.bind_vertex_buffers(0, Some((buffer_bundle.vertex_buffer.buffer.deref(), 0)));
        encoder.bind_index_buffer(IndexBufferView {
            buffer: &buffer_bundle.index_buffer.buffer,
            offset: 0,
            index_type: IndexType::U16,
        });

        let mut push_constants: [u32; VERTEX_PUSH_CONSTANTS_SIZE as usize] =
            [0; VERTEX_PUSH_CONSTANTS_SIZE as usize];
        push_constants[2] = game_world.camera_position.x.to_bits();
        push_constants[3] = game_world.camera_position.y.to_bits();
        push_constants[4] = game_world.camera_scale.to_bits();
        push_constants[5] = game_world.aspect_ratio.to_bits();

        for row in game_world.entities.iter_mut() {
            for entity in row.iter_mut() {
                let bits = entity.position.to_bits();
                push_constants[0] = bits[0];
                push_constants[1] = bits[1];

                encoder.push_graphics_constants(
                    &quad_pipeline.pipeline_layout,
                    ShaderStageFlags::VERTEX,
                    0,
                    &push_constants,
                );

                encoder.push_graphics_constants(
                    &quad_pipeline.pipeline_layout,
                    ShaderStageFlags::FRAGMENT,
                    mem::size_of::<u32>() as u32 * FRAG_PUSH_CONSTANTS_START,
                    &entity.state.to_color_bits(),
                );

                encoder.draw_indexed(0..6, 0, 0..1);
            }
        }

        Ok(())
    }

    pub unsafe fn draw_imgui<'a>(
        encoder: &mut RenderPassInlineEncoder<'_, I::Backend>,
        imgui_data: ImGuiDrawCommands<'_>,
        imgui_pipeline: &'a PipelineBundle<I::Backend>,
        buffer_bundle: &'a mut VertexIndexPairBufferBundle<I::Backend>,
        device: &<I::Backend as Backend>::Device,
        adapter: &Adapter<I::Backend>,
        descriptor_set: &<I::Backend as Backend>::DescriptorSet,
    ) -> Result<(), DrawingError> {
        buffer_bundle
            .update_size(
                (imgui_data.draw_data.total_vtx_count as usize * mem::size_of::<DrawVert>()) as u64,
                (imgui_data.draw_data.total_idx_count as usize * mem::size_of::<DrawIdx>()) as u64,
                &device,
                &adapter,
            )
            .map_err(|_| DrawingError::BufferCreationError)?;

        // Check our Buffers
        let VertexIndexPairBufferBundle {
            vertex_buffer: imgui_vertex_buffer,
            index_buffer: imgui_index_buffer,
        } = buffer_bundle;

        // Bind pipeline
        encoder.bind_graphics_pipeline(&imgui_pipeline.graphics_pipeline);

        // descriptor SET needs to be here...this is from the texture.
        encoder.bind_graphics_descriptor_sets(
            &imgui_pipeline.pipeline_layout,
            0,
            Some(descriptor_set),
            None as Option<u32>,
        );

        // Bind vertex and index buffers
        encoder.bind_vertex_buffers(0, Some((imgui_vertex_buffer.buffer.deref(), 0)));
        encoder.bind_index_buffer(buffer::IndexBufferView {
            buffer: &imgui_index_buffer.buffer,
            offset: 0,
            index_type: IndexType::U16,
        });

        // Set push constants
        #[rustfmt::skip]
        let push_constants: [u32; 4] = std::mem::transmute([
            // scale
            2.0 / imgui_data.imgui_dimensions.x,    2.0 / imgui_data.imgui_dimensions.y,
            //offset
            -1.0,           -1.0,
        ]);

        let viewport = Viewport {
            rect: Rect {
                x: 0,
                y: 0,
                w: imgui_data.imgui_dimensions.x as i16,
                h: imgui_data.imgui_dimensions.y as i16,
            },
            depth: 0.0..1.0,
        };
        encoder.set_viewports(0, &[viewport]);

        encoder.push_graphics_constants(
            &imgui_pipeline.pipeline_layout,
            ShaderStageFlags::VERTEX,
            0,
            &push_constants,
        );

        let mut vertex_offset = 0;
        let mut index_offset = 0;

        // Iterate over drawlists
        for list in imgui_data.draw_data.draw_lists() {
            // Update vertex and index buffers
            imgui_vertex_buffer.update_buffer(list.vtx_buffer(), vertex_offset);
            imgui_index_buffer.update_buffer(list.idx_buffer(), index_offset);

            for cmd in list.commands() {
                if let imgui::DrawCmd::Elements { count, cmd_params } = cmd {
                    // Calculate the scissor
                    let scissor = Rect {
                        x: cmd_params.clip_rect[0] as i16,
                        y: cmd_params.clip_rect[1] as i16,
                        w: (cmd_params.clip_rect[2] - cmd_params.clip_rect[0]) as i16,
                        h: (cmd_params.clip_rect[3] - cmd_params.clip_rect[1]) as i16,
                    };
                    encoder.set_scissors(0, &[scissor]);

                    // Actually draw things
                    encoder.draw_indexed(
                        index_offset as u32..(index_offset + count) as u32,
                        vertex_offset as i32,
                        0..1,
                    );

                    index_offset += count as usize;
                }
            }

            // Increment offsets
            vertex_offset += list.vtx_buffer().len();
        }

        Ok(())
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
            self.drop_swapchain();
            let (swapchain, backbuffer) = self
                .device
                .create_swapchain(&mut self.surface, swapchain_config, None)
                .map_err(|_| "Couldn't recreate the swapchain!")?;

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

            // Recreate the pipelines...
            self.pipeline_bundles.push(Self::create_pipeline(
                &mut self.device,
                &extent,
                &self.render_pass,
            )?);
            self.pipeline_bundles
                .push(Self::create_imgui_pipeline(&self.device, &self.render_pass)?);

            // Finally, we got ourselves a nice and shiny new swapchain!
            self.swapchain = manual_new!(swapchain);
            self.framebuffers = framebuffers;
            self.command_buffers = command_buffers;
            self.command_pool = manual_new!(command_pool);
        }
        Ok(())
    }

    fn drop_swapchain(&mut self) {
        self.device.wait_idle().unwrap();

        use core::ptr::read;
        unsafe {
            for framebuffer in self.framebuffers.drain(..) {
                self.device.destroy_framebuffer(framebuffer);
            }
            self.device
                .destroy_command_pool(manual_drop!(self.command_pool).into_raw());

            for pipeline_bundle in self.pipeline_bundles.drain(..) {
                pipeline_bundle.manually_drop(&self.device);
            }

            self.device.destroy_swapchain(manual_drop!(self.swapchain));
        }
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
            for this_pipeline in self.pipeline_bundles.drain(..) {
                this_pipeline.manually_drop(&self.device);
            }

            for this_bundled_bundle in self.vertex_index_buffer_bundles.drain(..) {
                this_bundled_bundle.manually_drop(&self.device);
            }

            // LAST RESORT STYLE CODE, NOT TO BE IMITATED LIGHTLY
            use core::ptr::read;
            self.device
                .destroy_command_pool(manual_drop!(self.command_pool).into_raw());
            self.device.destroy_render_pass(manual_drop!(self.render_pass));
            self.device.destroy_swapchain(manual_drop!(self.swapchain));

            ManuallyDrop::drop(&mut self.device);
            ManuallyDrop::drop(&mut self.instance);
        }
    }
}
