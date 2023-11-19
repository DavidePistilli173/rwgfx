use cgmath::{Point2, Vector2, Vector3};
use std::{error::Error, fmt};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::button::Button;
use crate::camera::Camera;
use crate::texture::Texture;
use crate::vertex;
use crate::vertex::Vertex;
use crate::{create_default_render_pipeline, shader};

/// Possible errors during window creation.
#[derive(Debug, Copy, Clone)]
pub enum AppCreationError {
    /// Error while creating the window.
    WindowCreation,
    /// Error while creating the rendering surface.
    SurfaceCreation,
    /// Error while retrieving a compatible rendering device (graphics card or other).
    NoPhysicalGraphicsDevice,
    /// Error while creating a logical rendering device or the command queue.
    DeviceOrQueueCreation,
}

impl Error for AppCreationError {}

impl fmt::Display for AppCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::WindowCreation => write!(f, "Failed to create the window."),
            Self::SurfaceCreation => write!(f, "Failed to create the rendering surface."),
            Self::NoPhysicalGraphicsDevice => {
                write!(f, "Failed to get a compatible physical rendering device.")
            }
            Self::DeviceOrQueueCreation => write!(
                f,
                "Failed to create a logical rendering device or a command queue."
            ),
        }
    }
}

pub struct App {
    /// Rendering surface.
    surface: wgpu::Surface,
    /// Graphics device.
    device: wgpu::Device,
    /// Command queue.
    queue: wgpu::Queue,
    /// Surface parameters.
    surface_config: wgpu::SurfaceConfiguration,
    /// Surface size.
    window_size: winit::dpi::PhysicalSize<u32>,
    /// Clear colour.
    clear_color: wgpu::Color,
    /// Vector of available rendering pipelines.
    render_pipelines: Vec<wgpu::RenderPipeline>,
    /// Index of the active rendering pipeline.
    active_render_pipeline: usize,
    /// Texture used for depth testing.
    depth_texture: Texture,
    /// Base camera.
    camera: Camera,
    /// Buttons.
    buttons: Vec<Button>,
    /// Logger.
    logger: rwlog::sender::Logger,
    // Window must be dropped after surface.
    window: Window,
}

impl App {
    fn create_default_render_pipelines(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        camera: &Camera,
    ) -> Vec<wgpu::RenderPipeline> {
        let mesh_uniform_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &shader::general::MeshUniform::layout_descriptor(),
                label: Some("mesh_bind_group_layout"),
            });

        let pipeline = create_default_render_pipeline!(
            &device,
            &surface_config,
            &[&camera.bind_group_layout(), &mesh_uniform_layout],
            &[&vertex::Plain::desc()]
        );

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/general.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("shader/general.wgsl render pipeline layout."),
                bind_group_layouts: &[camera.bind_group_layout(), &mesh_uniform_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shader/general.wgsl render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex::Plain::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        vec![render_pipeline]
    }

    /// Create a new application with default initialisation.
    pub fn new(logger: rwlog::sender::Logger) -> Result<Self, AppCreationError> {
        pollster::block_on(App::new_internal(logger))
    }

    /// Utility private function for actually creating the application.
    async fn new_internal(logger: rwlog::sender::Logger) -> Result<Self, AppCreationError> {
        // Necessary for wgpu error logging.
        env_logger::init();

        // Create a new event loop.
        let event_loop = EventLoop::new();

        // Create the window.
        let window = WindowBuilder::new().build(&event_loop).map_err(|err| {
            rwlog::rel_err!(&logger, "Failed to create window: {err}.");
            AppCreationError::WindowCreation
        })?;
        let window_size = window.inner_size();

        // Create the WGPU instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // Create the rendering surface.
        let surface = unsafe { instance.create_surface(&window) }.map_err(|err| {
            rwlog::rel_err!(&logger, "Failed to create window surface: {err}.");
            AppCreationError::SurfaceCreation
        })?;

        // Get the physical graphics device.
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| {
                rwlog::rel_err!(&logger, "Failed to get compatible graphics device.");
                AppCreationError::NoPhysicalGraphicsDevice
            })?;

        // Get logical device and command queue from the graphics adapter.
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .map_err(|err| {
                rwlog::rel_err!(
                    &logger,
                    "Failed to create logical graphics device and queue: {err}."
                );
                AppCreationError::DeviceOrQueueCreation
            })?;

        // Configure the surface.
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        // Create the depth texture.
        let depth_texture =
            Texture::create_depth_texture(&device, &surface_config, "depth_texture");

        // Create the camera.
        let camera = Camera::new_orthographic(
            &device,
            0.0,
            window_size.width as f32,
            0.0,
            window_size.height as f32,
            0.0,
            100.0,
        );

        // Create the default render pipelines.
        let render_pipelines =
            App::create_default_render_pipelines(&device, &surface_config, &camera);

        // Create a test button.
        let button = Button::new(
            &device,
            Point2::<f32> { x: 350.0, y: 250.0 },
            Vector2::<f32> { x: 100.0, y: 100.0 },
            -75.0,
            [0.5, 0.05, 0.05, 1.0],
        );
        let buttons = vec![button];

        Ok(Self {
            window,
            surface,
            device,
            queue,
            surface_config,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            render_pipelines,
            active_render_pipeline: 0,
            depth_texture,
            camera,
            buttons,
            logger,
            window_size,
        })
    }
}
