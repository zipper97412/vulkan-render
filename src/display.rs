
// However the Vulkan library doesn't provide any functionality to create and handle windows, as
// this would be out of scope. In order to open a window, we are going to use the `winit` crate.

// The `vulkano_win` crate is the link between `vulkano` and `winit`. Vulkano doesn't know about
// winit, and winit doesn't know about vulkano, so import a crate that will provide a link between
// the two.
use vulkano_win::{Window, VkSurfaceBuild};
use vulkano_win;
use winit;

use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::device::{Device, Queue};
use vulkano::swapchain::{Swapchain, SurfaceTransform};
use vulkano::image::swapchain::SwapchainImage;
use vulkano::pipeline::shader::ShaderModule;
use vulkano::device::DeviceExtensions;
use std::sync::Arc;


pub struct VulkanDisplay {
    instance: Instance,
    window: Window,
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<SwapchainImage>>,
    vertex_shader: ShaderModule<Arc<Device>>,
    fragment_shader: ShaderModule<Arc<Device>>,
    submissions: Vec<Box<GpuFuture>>,
    event_loop: winit::EventsLoop

}

impl VulkanDisplay {
    pub fn new() -> VulkanDisplay {
        let instance = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
        };

        let physical = PhysicalDevice::enumerate(&instance)
            .next().expect("no device available");

        let events_loop = winit::EventsLoop::new();
        let window = winit::WindowBuilder::new().build_vk_surface(&events_loop, &instance).unwrap();

        let queue = physical.queue_families().find(|q| {
            // We take the first queue that supports drawing to our window.
            q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false)
        }).expect("couldn't find a graphical queue family");

        let (device, mut queues) = {
            let device_ext = DeviceExtensions {
                khr_swapchain: true,
                .. DeviceExtensions::none()
            };

            Device::new(&physical, physical.supported_features(), &device_ext,
                        [(queue, 0.5)].iter().cloned()).expect("failed to create device")
        };

        let queue = queues.next().unwrap();

        let (swapchain, images) = {
            let caps = window.surface().get_capabilities(&physical)
                .expect("failed to get surface capabilities");

            let dimensions = caps.current_extent.unwrap_or([1280, 1024]);

            let present = caps.present_modes.iter().next().unwrap();

            let alpha = caps.supported_composite_alpha.iter().next().unwrap();

            let format = caps.supported_formats[0].0;

            Swapchain::new(&device, &window.surface(), caps.min_image_count, format, dimensions, 1,
                           &caps.supported_usage_flags, &queue, SurfaceTransform::Identity, alpha,
                           present, true, None).expect("failed to create swapchain")
        };
        mod vs { include!{concat!(env!("OUT_DIR"), "/shaders/src/triangle_vs.glsl")} }
        let vs = vs::Shader::load(&device).expect("failed to create shader module");
        mod fs { include!{concat!(env!("OUT_DIR"), "/shaders/src/triangle_fs.glsl")} }
        let fs = fs::Shader::load(&device).expect("failed to create shader module");

        let render_pass = Arc::new(single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: images[0].format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap());

        let pipeline = Arc::new(GraphicsPipeline::new(&device, GraphicsPipelineParams {
            vertex_input: SingleBufferDefinition::new(),
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
            multisample: Multisample::disabled(),
            fragment_shader: fs.main_entry_point(),
            depth_stencil: DepthStencil::disabled(),
            blend: Blend::pass_through(),
            render_pass: Subpass::from(render_pass.clone(), 0).unwrap(),
        }).unwrap());

        let framebuffers = images.iter().map(|image| {
            let attachments = render_pass.desc().start_attachments().color(image.clone());
            let dimensions = [image.dimensions()[0], image.dimensions()[1], 1];
            Framebuffer::new(render_pass.clone(), dimensions, attachments).unwrap()
        }).collect::<Vec<_>>();

        VulkanDisplay{
            instance,
            window,
            device,
            queue,
            swapchain,
            images,
            vertex_shader: vs,
            fragment_shader: fs,
            submissions: Vec::new(),
            event_loop
        }
    }

    pub

}