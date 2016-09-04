#[allow(dead_code, unused_parens)]

extern crate vulkano;
extern crate winit;
extern crate vulkano_win;

use vulkano_win::VkSurfaceBuild;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer;
use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::PrimaryCommandBufferBuilder;
use vulkano::command_buffer::Submission;
use vulkano::descriptor::pipeline_layout::EmptyPipeline;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::Subpass;
use vulkano::instance::Instance;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::GraphicsPipelineParams;
use vulkano::pipeline::blend::Blend;
use vulkano::pipeline::depth_stencil::DepthStencil;
use vulkano::pipeline::input_assembly::InputAssembly;
use vulkano::pipeline::multisample::Multisample;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::viewport::ViewportsState;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::viewport::Scissor;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::Swapchain;

use std::sync::Arc;
use std::time::Duration;

mod render_pass {
    use vulkano::format::Format;

    single_pass_renderpass! {
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: Format,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    }
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
impl_vertex!(Vertex, position);

pub struct Renderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    window: vulkano_win::Window,

    swapchain: Arc<Swapchain>,
    framebuffers: Vec<Arc<Framebuffer<render_pass::CustomRenderPass>>>,
    render_pass: Arc<render_pass::CustomRenderPass>,
    pipeline: Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>,
                                   EmptyPipeline,
                                   render_pass::CustomRenderPass>>,

    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
}

impl Renderer {
    pub fn new() -> Renderer {

        let instance = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
        };

        let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
            .next()
            .expect("no device available");

        println!("Using device: {} (type: {:?})",
                 physical.name(),
                 physical.ty());

        let window = winit::WindowBuilder::new().build_vk_surface(&instance).unwrap();

        let queue = physical.queue_families()
            .find(|q| {
                // We take the first queue that supports drawing to our window.
                q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false)
            })
            .expect("couldn't find a graphical queue family");

        let (device, mut queues) = {
            let device_ext = vulkano::device::DeviceExtensions {
                khr_swapchain: true,
                ..vulkano::device::DeviceExtensions::none()
            };

            Device::new(&physical,
                        physical.supported_features(),
                        &device_ext,
                        [(queue, 0.5)].iter().cloned())
                .expect("failed to create device")
        };

        let queue = queues.next().unwrap();

        let (swapchain, images) = {
            let caps = window.surface()
                .get_capabilities(&physical)
                .expect("failed to get surface capabilities");

            let dimensions = caps.current_extent.unwrap_or([1280, 1024]);

            let present = caps.present_modes.iter().next().unwrap();

            let alpha = caps.supported_composite_alpha.iter().next().unwrap();

            let format = caps.supported_formats[0].0;

            Swapchain::new(&device,
                           &window.surface(),
                           2,
                           format,
                           dimensions,
                           1,
                           &caps.supported_usage_flags,
                           &queue,
                           SurfaceTransform::Identity,
                           alpha,
                           present,
                           true,
                           None)
                .expect("failed to create swapchain")
        };

        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(&device,
                                           &BufferUsage::all(),
                                           Some(queue.family()),
                                           [Vertex { position: [-0.5, -0.25] },
                                            Vertex { position: [0.0, 0.5] },
                                            Vertex { position: [0.25, -0.1] }]
                                               .iter()
                                               .cloned())
                .expect("failed to create buffer")
        };

        mod vs {
            include!{concat!(env!("OUT_DIR"), "/shaders/shaders/triangle_vs.glsl")}
        }
        let vs = vs::Shader::load(&device).expect("failed to create shader module");
        mod fs {
            include!{concat!(env!("OUT_DIR"), "/shaders/shaders/triangle_fs.glsl")}
        }
        let fs = fs::Shader::load(&device).expect("failed to create shader module");

        let render_pass =
            render_pass::CustomRenderPass::new(&device,
                                               &render_pass::Formats {
                                                   // Use the format of the images and one sample.
                                                   color: (images[0].format(), 1),
                                               })
                .unwrap();

        let vertex_input: SingleBufferDefinition<Vertex> = SingleBufferDefinition::new();

        let pipeline = GraphicsPipeline::new(&device, GraphicsPipelineParams {
            vertex_input: vertex_input,
            vertex_shader: vs.main_entry_point(),
            input_assembly: InputAssembly::triangle_list(),
            tessellation: None,
            geometry_shader: None,
            viewport: ViewportsState::Fixed {
                data: vec![(
                    Viewport {
                        origin: [0.0, 0.0],
                        depth_range: 0.0 .. 1.0,
                        dimensions: [images[0].dimensions()[0] as f32,
                                     images[0].dimensions()[1] as f32],
                    },
                    Scissor::irrelevant()
                )],
            },

            raster: Default::default(),

            // If we use multisampling, we can pass additional configuration.
            multisample: Multisample::disabled(),

            // See `vertex_shader`.
            fragment_shader: fs.main_entry_point(),

            depth_stencil: DepthStencil::disabled(),

            blend: Blend::pass_through(),

            layout: &EmptyPipeline::new(&device).unwrap(),

            render_pass: Subpass::from(&render_pass, 0).unwrap(),
        }).unwrap();

        let framebuffers = images.iter()
            .map(|image| {
                let dimensions = [image.dimensions()[0], image.dimensions()[1], 1];
                Framebuffer::new(&render_pass,
                                 dimensions,
                                 render_pass::AList { color: image })
                    .unwrap()
            })
            .collect::<Vec<_>>();

        return Renderer {
            swapchain: swapchain,
            window: window,
            device: device,
            queue: queue,
            render_pass: render_pass,
            framebuffers: framebuffers,
            pipeline: pipeline,
            vertex_buffer: vertex_buffer,
        };
    }

    pub fn run(&self) {
        let mut submissions: Vec<Arc<Submission>> = Vec::new();

        loop {
            submissions.retain(|s| s.destroying_would_block());
            let image_num = self.swapchain.acquire_next_image(Duration::new(1, 0)).unwrap();
            let command_buffer = PrimaryCommandBufferBuilder::new(&self.device,
                                                                  self.queue.family())
                .draw_inline(&self.render_pass,
                             &self.framebuffers[image_num],
                             render_pass::ClearValues { color: [0.0, 0.0, 1.0, 1.0] })
                .draw(&self.pipeline,
                      &self.vertex_buffer,
                      &DynamicState::none(),
                      (),
                      &())
                .draw_end()
                .build();

            submissions.push(command_buffer::submit(&command_buffer, &self.queue).unwrap());
            self.swapchain.present(&self.queue, image_num).unwrap();

            for ev in self.window.window().poll_events() {
                match ev {
                    winit::Event::Closed => break,
                    _ => (),
                }
            }
        }

    }
}
