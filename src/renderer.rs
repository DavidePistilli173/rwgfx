//! Main rendering context.

use cgmath::Vector2;
use std::cell::RefCell;
use std::collections::hash_map;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;
use wgpu::SurfaceError;

use crate::asset;
use crate::camera::Camera;
use crate::drawable::Drawable;
use crate::error::{RenderError, RendererCreationError};
use crate::texture::{self, Texture};
use crate::vertex::Vertex;
use crate::{create_default_render_pipeline, pipeline, shader, text, vertex};

/// Structure containing data and functions for rendering the current frame.
#[derive(Debug)]
pub struct FrameCtx<'pass> {
    /// GPU compute context.
    ctx: &'pass rwcompute::Context,
    /// Main camera for the active frame.
    camera: &'pass Camera,
    /// Render pass for the active frame.
    render_pass: RefCell<wgpu::RenderPass<'pass>>,
    /// ID of the currently active pipeline.
    active_pipeline_id: u64,
}

impl<'pass> FrameCtx<'pass> {
    /// Get the active pipeline ID.
    pub fn active_pipeline_id(&self) -> u64 {
        self.active_pipeline_id
    }

    /// Bind data to the active render pass.
    pub fn bind_data<'data>(&self, index: u32, data: &'data wgpu::BindGroup)
    where
        'data: 'pass,
    {
        self.render_pass
            .borrow_mut()
            .set_bind_group(index, data, &[])
    }

    /// Draw a vertex buffer following previously set index buffer.
    pub fn draw_indexed(&self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.render_pass
            .borrow_mut()
            .draw_indexed(indices, base_vertex, instances);
    }

    /// Get the GPU context.
    pub fn gpu_ctx(&self) -> &rwcompute::Context {
        self.ctx
    }

    /// Set the index buffer for the next draw call.
    pub fn set_index_buffer<'data>(
        &self,
        slice: wgpu::BufferSlice<'data>,
        format: wgpu::IndexFormat,
    ) where
        'data: 'pass,
    {
        self.render_pass
            .borrow_mut()
            .set_index_buffer(slice, format);
    }

    /// Set the rendering pipeline.
    pub fn set_pipeline(&mut self, id: u64, pipeline: &'pass wgpu::RenderPipeline) {
        // Bind the next pipeline and return true.
        self.render_pass.borrow_mut().set_pipeline(pipeline);
        self.active_pipeline_id = id;
    }

    /// Assigns a vertex buffer slice to a given slot.
    pub fn set_vertex_buffer<'data>(&self, slot: u32, slice: wgpu::BufferSlice<'data>)
    where
        'data: 'pass,
    {
        self.render_pass.borrow_mut().set_vertex_buffer(slot, slice);
    }
}

/// All data and code for a rendering context.
pub struct Renderer {
    /// Rendering surface.
    surface: wgpu::Surface,
    /// GPU computing context.
    ctx: rwcompute::Context,
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
    /// Logger.
    logger: rwlog::sender::Logger,
}

impl Renderer {
    /// Get the GPU compute context.
    pub fn ctx(&self) -> &rwcompute::Context {
        &self.ctx
    }

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
        let general_texture_layout =
            Texture::bind_group_layout(device, texture::TextureFormat::Rgba8UnormSrgb);

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
                &general_texture_layout
            ],
            &[vertex::Textured::desc()]
        );

        let text_texture_layout = Texture::bind_group_layout(device, text::TEXTURE_FORMAT);
        let text_shader = device.create_shader_module(wgpu::include_wgsl!("shader/text.wgsl"));
        let text_pipeline = create_default_render_pipeline!(
            &device,
            &surface_config,
            "shader/text.wgsl",
            text_shader,
            &[
                &camera.bind_group_layout(),
                &mesh_uniform_layout,
                &text_texture_layout
            ],
            &[vertex::Textured::desc()]
        );

        let mut render_pipelines = HashMap::new();
        render_pipelines.insert(pipeline::ID_GENERAL, general_pipeline);
        render_pipelines.insert(pipeline::ID_TEXT, text_pipeline);

        render_pipelines
    }

    /// Get the graphics device that this context is using.
    pub fn device(&self) -> &wgpu::Device {
        &self.ctx.device()
    }

    pub fn draw<'pass>(
        &'pass mut self,
        asset_manager: &'pass mut asset::Manager,
        entities: &'pass [&impl Drawable],
    ) -> Result<(), RenderError> {
        let output_surface = self
            .surface
            .get_current_texture()
            .map_err(|err| match err {
                SurfaceError::Lost => RenderError::SurfaceInvalid,
                SurfaceError::Outdated => RenderError::SurfaceInvalid,
                SurfaceError::OutOfMemory => RenderError::OutOfMemory,
                SurfaceError::Timeout => RenderError::GraphicsDeviceNotResponding,
            })?;

        let output_view = output_surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder =
            self.ctx
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // Update the camera.
        self.camera.update_gpu_data(&self.ctx.queue());

        asset_manager
            .text_handler_mut()
            .resize_caches(&self.logger, &self.ctx)
            .unwrap_or_else(|err| {
                rwlog::err!(&self.logger, "Failed to resize font caches: {err}.");
            });

        let render_pass = RefCell::new(command_encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
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
            },
        ));

        let mut frame_ctx = FrameCtx {
            ctx: &self.ctx(),
            camera: &self.camera,
            render_pass,
            active_pipeline_id: 0,
        };

        for (id, pipeline) in self.render_pipelines.iter() {
            frame_ctx.set_pipeline(*id, pipeline);
            frame_ctx.bind_data(0, self.camera.bind_group());

            for entity in entities {
                entity.draw(&frame_ctx, &asset_manager);
            }
        }

        std::mem::drop(frame_ctx); // Terminate the render pass.

        self.ctx()
            .queue()
            .submit(std::iter::once(command_encoder.finish()));

        output_surface.present();

        Ok(())
    }

    /// Create a new application with default initialisation.
    pub fn new<W>(
        logger: rwlog::sender::Logger,
        window: &W,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, RendererCreationError>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        pollster::block_on(Renderer::new_internal(
            logger,
            window,
            window_width,
            window_height,
        ))
    }

    /// Utility private function for actually creating the application.
    async fn new_internal<W>(
        logger: rwlog::sender::Logger,
        window: &W,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, RendererCreationError>
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
            rwlog::err!(&logger, "Failed to create window surface: {err}.");
            RendererCreationError::SurfaceCreation
        })?;

        // Create the graphics compute context.
        let ctx = rwcompute::Context::new(
            logger.clone(),
            Some(instance),
            Some(&surface),
            wgpu::Features::empty(),
        )
        .map_err(|err| {
            rwlog::err!(
                &logger,
                "Failed to create the graphics compute context: {err}."
            );
            RendererCreationError::GraphicsContextCreation
        })?;

        // Configure the surface.
        let surface_capabilities = surface.get_capabilities(&ctx.adapter());
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
        surface.configure(&ctx.device(), &surface_config);

        // Create the depth texture.
        let depth_texture =
            Texture::create_depth_texture(&ctx.device(), &surface_config, "depth_texture");

        // Create the camera.
        let camera: Camera = Camera::new_orthographic(
            &ctx.device(),
            0.0,
            window_width as f32,
            0.0,
            window_height as f32,
            0.0,
            100.0,
        );

        // Create the default render pipelines.
        let render_pipelines =
            Renderer::create_default_render_pipelines(&ctx.device(), &surface_config, &camera);

        Ok(Self {
            surface,
            ctx,
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
            logger,
            window_size: Vector2::<u32> {
                x: window_width,
                y: window_height,
            },
        })
    }

    /// Resize the output surface.
    pub fn resize(&mut self, window_width: u32, window_height: u32) {
        if window_width > 0 && window_height > 0 {
            self.window_size = Vector2::<u32> {
                x: window_width,
                y: window_height,
            };
            self.surface_config.width = window_width;
            self.surface_config.height = window_height;
            self.surface
                .configure(&self.ctx.device(), &self.surface_config);
            self.depth_texture = Texture::create_depth_texture(
                &self.ctx.device(),
                &self.surface_config,
                "depth_texture",
            );
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
