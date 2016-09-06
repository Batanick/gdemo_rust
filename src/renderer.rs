extern crate vulkano;
extern crate winit;
extern crate vulkano_win;
extern crate time;

extern crate cgmath;

use vulkano_win::VkSurfaceBuild;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer;
use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::PrimaryCommandBufferBuilder;
use vulkano::command_buffer::Submission;
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

use camera::Camera;
use window_state::WindowState;

mod vs {
    include!{concat!(env!("OUT_DIR"), "/shaders/shaders/main_vs.glsl")}
}
mod fs {
    include!{concat!(env!("OUT_DIR"), "/shaders/shaders/main_fs.glsl")}
}

mod pipeline_layout {
    pipeline_layout!{
            set0: {
                uniforms: UniformBuffer<super::super::vs::ty::Data>
            }
        }
}

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
    position: [f32; 3],
    color: [f32; 3],
}
impl_vertex!(Vertex, position, color);

pub struct Renderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    window: vulkano_win::Window,

    swapchain: Arc<Swapchain>,
    framebuffers: Vec<Arc<Framebuffer<render_pass::CustomRenderPass>>>,
    render_pass: Arc<render_pass::CustomRenderPass>,

    pipeline_layout: Arc<pipeline_layout::CustomPipeline>,
    pipeline: Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>,
                                   pipeline_layout::CustomPipeline,
                                   render_pass::CustomRenderPass>>,

    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    uniform_buffer: Arc<CpuAccessibleBuffer<vs::ty::Data>>,

    camera: Camera,
    window_state: WindowState,
    submissions: Vec<Arc<Submission>>,
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
                                           [Vertex {
                                                position: [-0.5, -0.25, 0.0],
                                                color: [1.0, 0.0, 0.0],
                                            },
                                            Vertex {
                                                position: [0.0, 0.5, 0.0],
                                                color: [0.0, 1.0, 0.0],
                                            },
                                            Vertex {
                                                position: [0.25, -0.1, 0.0],
                                                color: [0.0, 0.0, 1.0],
                                            }]
                                               .iter()
                                               .cloned())
                .expect("failed to create buffer")
        };

        let proj = cgmath::perspective(cgmath::Rad(3.141592 / 2.0),
                                       {
                                           let d = images[0].dimensions();
                                           d[0] as f32 / d[1] as f32
                                       },
                                       0.01,
                                       100.0);
        let view = cgmath::Matrix4::look_at(cgmath::Point3::new(0.3, 0.3, 1.0),
                                            cgmath::Point3::new(0.0, 0.0, 0.0),
                                            cgmath::Vector3::new(0.0, -1.0, 0.0));
        let scale = cgmath::Matrix4::from_scale(0.01);

        let uniform_buffer = CpuAccessibleBuffer::from_data(&device,
                                                            &vulkano::buffer::BufferUsage::all(),
                                                            Some(queue.family()),
                                                            vs::ty::Data {
                                                                worldview: (view * scale).into(),
                                                                proj: proj.into(),
                                                            })
            .expect("failed to create buffer");

        let vs = vs::Shader::load(&device).expect("failed to create shader module");
        let fs = fs::Shader::load(&device).expect("failed to create shader module");

        let render_pass =
            render_pass::CustomRenderPass::new(&device,
                                               &render_pass::Formats {
                                                   // Use the format of the images and one sample.
                                                   color: (images[0].format(), 1),
                                               })
                .unwrap();

        let vertex_input: SingleBufferDefinition<Vertex> = SingleBufferDefinition::new();

        let pipeline_layout = pipeline_layout::CustomPipeline::new(&device).unwrap();

        let pipeline = GraphicsPipeline::new(&device,
                                             GraphicsPipelineParams {
                                                 vertex_input: vertex_input,
                                                 vertex_shader: vs.main_entry_point(),
                                                 input_assembly: InputAssembly::triangle_list(),
                                                 tessellation: None,
                                                 geometry_shader: None,
                                                 viewport: ViewportsState::Fixed {
                                                     data: vec![(Viewport {
                                                         origin: [0.0, 0.0],
                                                         depth_range: 0.0..1.0,
                                                         dimensions:
                                                             [images[0].dimensions()[0] as f32,
                                                              images[0].dimensions()[1] as f32],
                                                     },
                                                      Scissor::irrelevant())],
                                                 },

                                                 raster: Default::default(),
                                                 multisample: Multisample::disabled(),
                                                 fragment_shader: fs.main_entry_point(),
                                                 depth_stencil: DepthStencil::disabled(),
                                                 blend: Blend::pass_through(),
                                                 layout: &pipeline_layout,
                                                 render_pass: Subpass::from(&render_pass, 0)
                                                     .unwrap(),
                                             })
            .unwrap();

        let framebuffers = images.iter()
            .map(|image| {
                let dimensions = [image.dimensions()[0], image.dimensions()[1], 1];
                Framebuffer::new(&render_pass,
                                 dimensions,
                                 render_pass::AList { color: image })
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let window_state = WindowState::new(window.window());
        Renderer {
            swapchain: swapchain,
            window: window,
            device: device,
            queue: queue,
            render_pass: render_pass,
            framebuffers: framebuffers,
            pipeline: pipeline,
            pipeline_layout: pipeline_layout,
            vertex_buffer: vertex_buffer,
            uniform_buffer: uniform_buffer,

            camera: Camera::new(),
            window_state: window_state,

            submissions: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        let mut focused = true;
        let mut time_delta;
        let mut last_tick_time = time::precise_time_ns();

        loop {
            let current_time = time::precise_time_ns();
            time_delta = ((current_time - last_tick_time) / 1000000) as f32;

            if focused {
                self.camera.update(&self.window_state, time_delta);
                self.window.window().set_title(self.camera.get_pos());

                let proj = cgmath::perspective(cgmath::Rad(3.141592 / 2.0),
                                               self.window_state.get_aspect(),
                                               0.01,
                                               100.0);
                let view = self.camera.get_view();

                self.uniform_buffer.write(Duration::from_millis(0)).map(|oldData| {
                    vs::ty::Data {
                        worldview: view.into(),
                        proj: proj.into(),
                    }
                });
                self.render();
            }

            for ev in self.window.window().poll_events() {
                match ev {
                    winit::Event::Closed => break,
                    winit::Event::KeyboardInput(state, _, key_code) => {
                        self.window_state
                            .switch(key_code.unwrap_or(winit::VirtualKeyCode::NoConvert),
                                    state == winit::ElementState::Pressed)
                    }
                    winit::Event::MouseMoved((mouse_x, mouse_y)) => {
                        self.window_state.update_mouse(mouse_x, mouse_y)
                    }
                    winit::Event::Resized(width, height) => {
                        self.window_state.update_size(width, height)
                    }
                    winit::Event::Focused(focus) => focused = focus,
                    _ => (),
                }
            }

            let size = self.window_state.get_window_size();
            self.window
                .window()
                .set_cursor_position((size.0 / 2) as i32, (size.1 / 2) as i32)
                .expect("Unable to update cursor position");

            last_tick_time = current_time;
        }
    }

    fn render(&mut self) {
        self.submissions.retain(|s| s.destroying_would_block());
        let descriptor_pool =
            vulkano::descriptor::descriptor_set::DescriptorPool::new(&self.device);

        let set = pipeline_layout::set0::Set::new(&descriptor_pool,
                                                  &self.pipeline_layout,
                                                  &pipeline_layout::set0::Descriptors {
                                                      uniforms: &self.uniform_buffer,
                                                  });

        let image_num = self.swapchain.acquire_next_image(Duration::new(1, 0)).unwrap();
        let command_buffer = PrimaryCommandBufferBuilder::new(&self.device, self.queue.family())
            .draw_inline(&self.render_pass,
                         &self.framebuffers[image_num],
                         render_pass::ClearValues { color: [0.0, 0.0, 1.0, 1.0] })
            .draw(&self.pipeline,
                  &self.vertex_buffer,
                  &DynamicState::none(),
                  &set,
                  &())
            .draw_end()
            .build();

        self.submissions.push(command_buffer::submit(&command_buffer, &self.queue).unwrap());
        self.swapchain.present(&self.queue, image_num).unwrap();
    }
}
