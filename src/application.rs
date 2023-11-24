//! Main GUI application.

use cgmath::{Point2, Vector2};
use std::collections::HashMap;
use std::{error::Error, fmt};
use winit::event::{self, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use crate::button::Button;
use crate::camera::Camera;
use crate::texture::Texture;
use crate::vertex::Vertex;
use crate::{create_default_render_pipeline, shader};
use crate::{pipelines, vertex};

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

/// All data and code for a GUI application.
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
    /// Map of available rendering pipelines ordered by ID.
    render_pipelines: HashMap<u64, wgpu::RenderPipeline>,
    /// Texture used for depth testing.
    depth_texture: Texture,
    /// Base camera.
    camera: Camera,
    /// Buttons.
    buttons: Vec<Button>,
    /// Logger.
    logger: rwlog::sender::Logger,
    /// Last time the main loop updated the application.
    last_update_time: chrono::DateTime<chrono::Local>,
    /// Main event loop of the window.
    event_loop: Option<EventLoop<()>>,
    /// Window must be dropped after surface.
    window: Window,
}

impl App {
    fn create_default_render_pipelines(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        camera: &Camera,
    ) -> HashMap<u64, wgpu::RenderPipeline> {
        let mesh_uniform_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &shader::general::MeshUniform::layout_descriptor(),
                label: Some("mesh_bind_group_layout"),
            });

        let general_shader =
            device.create_shader_module(wgpu::include_wgsl!("shader/general.wgsl"));
        let general_pipeline = create_default_render_pipeline!(
            &device,
            &surface_config,
            "shader/general.wgsl",
            general_shader,
            &[&camera.bind_group_layout(), &mesh_uniform_layout],
            &[vertex::Plain::desc()]
        );

        let mut render_pipelines = HashMap::new();
        render_pipelines.insert(pipelines::ID_GENERAL, general_pipeline);

        render_pipelines
    }

    /// Propagate a window event to all widgets of the window.
    /// If the event was consumed, returns true, otherwise false.
    fn propagate_event(&mut self, event: &WindowEvent) -> bool {
        //todo!();
        false
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
            depth_texture,
            camera,
            buttons,
            logger,
            last_update_time: chrono::Local::now(),
            window_size,
            event_loop: Some(event_loop),
        })
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Render pass.
        {
            // Initialise the render pass.
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // Iterate through all pipelines.
            for (id, pipeline) in self.render_pipelines.iter() {
                render_pass.set_pipeline(&pipeline);
                render_pass.set_bind_group(0, self.camera.bind_group(), &[]);

                for button in self.buttons.iter() {
                    button.draw(&mut render_pass);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.surface_config, "depth_texture");
        }
    }

    fn update(&mut self) {
        let current_time = chrono::Local::now();
        let delta_time = current_time - self.last_update_time;
        self.last_update_time = current_time;

        for button in self.buttons.iter_mut() {
            button.update(&delta_time);
        }
    }
}

/// Run the main loop of the application.
pub fn run(mut app: App) {
    if let Some(event_loop) = app.event_loop.take() {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            // Process incoming events.
            match event {
                Event::WindowEvent {
                    window_id,
                    ref event,
                } => {
                    if window_id == app.window.id() && !app.propagate_event(&event) {
                        match event {
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            WindowEvent::Resized(physical_size) => app.resize(*physical_size),
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                app.resize(**new_inner_size)
                            }
                            _ => (),
                        }
                    }
                }
                Event::RedrawRequested(window_id) if window_id == app.window.id() => {
                    match app.render() {
                        Ok(_) => (),
                        Err(wgpu::SurfaceError::Lost) => app.resize(app.window_size),
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            rwlog::rel_err!(&app.logger, "Not enough GPU memory!");
                            *control_flow = ControlFlow::Exit;
                        }
                        Err(e) => {
                            rwlog::warn!(&app.logger, "{e}");
                        }
                    };
                }
                Event::MainEventsCleared => {
                    app.window.request_redraw();
                }
                _ => (),
            }

            // Update the application.
            app.update();
        });
    } else {
        rwlog::rel_fatal!(&app.logger, "Event loop not initialised.");
    }
}
