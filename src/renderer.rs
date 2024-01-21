//! Main rendering context.

use cgmath::Vector2;
use std::collections::HashMap;
use wgpu::SurfaceError;

use crate::asset;
use crate::camera::Camera;
use crate::error::{ContextCreationError, RenderError, RendererCreationError};
use crate::text;
use crate::texture::{self, Texture};
use crate::vertex::Vertex;
use crate::{create_default_render_pipeline, shader};
use crate::{pipeline, vertex};

/// Data of the current frame rendering.
pub struct FrameContext<'a> {
    /// ID of the pipeline currently used for drawing.
    pub pipeline_id: u64,
    /// Graphics device used for the current frame.
    pub device: &'a wgpu::Device,
    /// Command queue used for the current frame.
    pub queue: &'a wgpu::Queue,
    /// Size of the rendered window.
    pub window_size: Vector2<u32>,
    /// Graphics asset manager.
    pub asset_manager: &'a asset::Manager,
}

/// All data and code for a rendering context.
pub struct Renderer {
    /// Rendering surface.
    surface: wgpu::Surface,
    /// Graphics device.
    device: wgpu::Device,
    /// Command queue.
    queue: wgpu::Queue,
    /// Surface parameters.
    surface_config: wgpu::SurfaceConfiguration,
    /// Surface size.
    window_size: Vector2<u32>,
    /// Clear colour.
    clear_color: wgpu::Color,
    /// Map of available rendering pipelines ordered by ID.
    render_pipelines: HashMap<u64, wgpu::RenderPipeline>,
    /// Texture used for depth testing.
    depth_texture: Texture,
    /// Base camera.
    camera: Camera,
    /// Graphics asset manager.
    asset_manager: asset::Manager,
    /// Font library
    font_library: freetype::library::Library,
    /// Logger.
    logger: rwlog::sender::Logger,
}

impl Renderer {
    /// Create the default rendering pipelines.
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
        let texture_layout = Texture::bind_group_layout(device);

        let general_shader =
            device.create_shader_module(wgpu::include_wgsl!("shader/general.wgsl"));
        let general_pipeline = create_default_render_pipeline!(
            &device,
            &surface_config,
            "shader/general.wgsl",
            general_shader,
            &[
                &camera.bind_group_layout(),
                &mesh_uniform_layout,
                &texture_layout
            ],
            &[vertex::Textured::desc()]
        );

        let mut render_pipelines = HashMap::new();
        render_pipelines.insert(pipeline::ID_GENERAL, general_pipeline);

        render_pipelines
    }

    /// Create the asset manager and load the default assets.
    fn create_default_assets(
        logger: &rwlog::sender::Logger,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_library: &freetype::library::Library,
        include_default_textures: bool,
    ) -> asset::Manager {
        // Load textures.
        let empty_data = include_bytes!("texture/empty.png");

        let mut asset_manager = asset::Manager::new(logger.clone());

        if !asset_manager.load_texture_from_bytes(
            device,
            queue,
            empty_data,
            texture::ID_EMPTY,
            "empty",
        ) {
            rwlog::rel_fatal!(&logger, "Failed to embedded empty texture.");
        }

        if include_default_textures {
            let hamburger_data = include_bytes!("texture/hamburger.png");
            if !asset_manager.load_texture_from_bytes(
                device,
                queue,
                hamburger_data,
                texture::ID_HAMBURGER,
                "hamburger",
            ) {
                rwlog::rel_err!(&logger, "Failed to load embedded hamburger texture.");
            }
        }

        // Load fonts.
        if !asset_manager.load_font_from_file(
            &font_library,
            "font/gnu-free=font/FreeMono.ttf",
            text::ID_DEFAULT,
            &logger,
        ) {
            rwlog::fatal!(&logger, "Failed to load the default font.");
        }

        asset_manager
    }

    /// Get the graphics device that this context is using.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Create a new application with default initialisation.
    pub fn new<W>(
        logger: rwlog::sender::Logger,
        window: &W,
        window_width: u32,
        window_height: u32,
        include_default_textures: bool,
    ) -> Result<Self, ContextCreationError>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        pollster::block_on(Renderer::new_internal(
            logger,
            window,
            window_width,
            window_height,
            include_default_textures,
        ))
    }

    /// Utility private function for actually creating the application.
    async fn new_internal<W>(
        logger: rwlog::sender::Logger,
        window: &W,
        window_width: u32,
        window_height: u32,
        include_default_textures: bool,
    ) -> Result<Self, ContextCreationError>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        // Necessary for wgpu error logging.
        env_logger::init();

        // Create the WGPU instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // Create the rendering surface.
        let surface = unsafe { instance.create_surface(&window) }.map_err(|err| {
            rwlog::rel_err!(&logger, "Failed to create window surface: {err}.");
            ContextCreationError::SurfaceCreation
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
                ContextCreationError::NoPhysicalGraphicsDevice
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
                ContextCreationError::DeviceOrQueueCreation
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
            width: window_width,
            height: window_height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        // Create the depth texture.
        let depth_texture =
            Texture::create_depth_texture(&device, &surface_config, "depth_texture");

        // Create the font library.
        let font_library = freetype::library::Library::init().map_err(|err| {
            rwlog::err!(&logger, "Failed to initialise the font library: {err}.");
            RendererCreationError::FontLibraryCreation
        })?;

        // Create the asset manager and load the default assets.
        let asset_manager = Renderer::create_default_assets(
            &logger,
            &device,
            &queue,
            &font_library,
            include_default_textures,
        );

        // Create the camera.
        let camera = Camera::new_orthographic(
            &device,
            0.0,
            window_width as f32,
            0.0,
            window_height as f32,
            0.0,
            100.0,
        );

        // Create the default render pipelines.
        let render_pipelines =
            Renderer::create_default_render_pipelines(&device, &surface_config, &camera);

        Ok(Self {
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
            asset_manager,
            camera,
            logger,
            font_library,
            window_size: Vector2::<u32> {
                x: window_width,
                y: window_height,
            },
        })
    }

    pub fn render<'a, F>(&'a mut self, draw_calls: F) -> Result<(), RenderError>
    where
        F: for<'b> Fn(&mut wgpu::RenderPass<'b>, &'a mut FrameContext<'a>, [&'b &'a (); 0]),
    {
        let output = self
            .surface
            .get_current_texture()
            .map_err(|err| match err {
                SurfaceError::Lost => RenderError::SurfaceInvalid,
                SurfaceError::Outdated => RenderError::SurfaceInvalid,
                SurfaceError::OutOfMemory => RenderError::OutOfMemory,
                SurfaceError::Timeout => RenderError::GraphicsDeviceNotResponding,
            })?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Update the camera.
        self.camera.update_gpu_data(&self.queue);

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
            let mut frame_context = FrameContext {
                pipeline_id: pipeline::ID_INVALID,
                device: &self.device,
                queue: &self.queue,
                window_size: self.window_size,
                asset_manager: &self.asset_manager,
            };

            for (id, pipeline) in self.render_pipelines.iter() {
                frame_context.pipeline_id = *id;
                render_pass.set_pipeline(&pipeline);
                render_pass.set_bind_group(0, self.camera.bind_group(), &[]);

                draw_calls(&mut render_pass, &mut frame_context, []);
            }
        }

        // Submit the rendering queue and present the output image.
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, window_width: u32, window_height: u32) {
        if window_width > 0 && window_height > 0 {
            self.window_size = Vector2::<u32> {
                x: window_width,
                y: window_height,
            };
            self.surface_config.width = window_width;
            self.surface_config.height = window_height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.surface_config, "depth_texture");
            self.camera.rebuild_orthographic(
                0.0,
                self.window_size.x as f32,
                0.0,
                self.window_size.y as f32,
                0.0,
                100.0,
            );
        }
    }
}
