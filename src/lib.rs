use cgmath::{Point2, Vector2, Vector3};
use image::GenericImageView;
use shader::general::{CameraUniform, MeshUniform};
use texture::Texture;
use wgpu::util::DeviceExt;
use winit::{
    event::{self, *},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub mod animation;
pub mod application;
pub mod button;
pub mod camera;
pub mod pipelines;
pub mod shader;
mod texture;
pub mod vertex;

use button::Button;
use camera::Camera;
use vertex::Vertex;

const VERTICES: &[vertex::Textured] = &[
    vertex::Textured {
        position: [-0.0868241, 0.49240386],
        tex_coords: [0.4131759, 0.00759614],
    }, // A
    vertex::Textured {
        position: [-0.49513406, 0.06958647],
        tex_coords: [0.0048659444, 0.43041354],
    }, // B
    vertex::Textured {
        position: [-0.21918549, -0.44939706],
        tex_coords: [0.28081453, 0.949397],
    }, // C
    vertex::Textured {
        position: [0.35966998, -0.3473291],
        tex_coords: [0.85967, 0.84732914],
    }, // D
    vertex::Textured {
        position: [0.44147372, 0.2347359],
        tex_coords: [0.9414737, 0.2652641],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

const VERTICES2: &[vertex::Textured] = &[
    vertex::Textured {
        position: [0.0, 0.5],
        tex_coords: [1.0, 0.0],
    }, // A
    vertex::Textured {
        position: [0.2, 0.2],
        tex_coords: [1.0, 1.0],
    }, // B
    vertex::Textured {
        position: [-0.2, 0.2],
        tex_coords: [1.0, 1.0],
    }, // C
    vertex::Textured {
        position: [0.5, 0.2],
        tex_coords: [1.0, 0.0],
    }, // D
    vertex::Textured {
        position: [0.3, -0.2],
        tex_coords: [1.0, 1.0],
    }, // E
    vertex::Textured {
        position: [0.4, -0.5],
        tex_coords: [1.0, 0.0],
    }, // F
    vertex::Textured {
        position: [0.0, -0.2],
        tex_coords: [1.0, 1.0],
    }, // G
    vertex::Textured {
        position: [-0.4, -0.5],
        tex_coords: [1.0, 0.0],
    }, // H
    vertex::Textured {
        position: [-0.3, -0.2],
        tex_coords: [1.0, 1.0],
    }, // I
    vertex::Textured {
        position: [-0.5, 0.2],
        tex_coords: [1.0, 0.0],
    }, // L
];

const INDICES2: &[u16] = &[
    2, 1, 0, 1, 4, 3, 6, 5, 4, 8, 7, 6, 2, 9, 8, 8, 6, 2, 6, 1, 2, 6, 4, 1,
];

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    render_pipeline_2: wgpu::RenderPipeline,
    active_pipeline: u8,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_buffer_2: wgpu::Buffer,
    index_buffer_2: wgpu::Buffer,
    active_mesh: u8,
    diffuse_texture: Texture,
    diffuse_bind_group: wgpu::BindGroup,
    depth_texture: Texture,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    button: Button,
    logger: rwlog::sender::Logger,
    // Window must be dropped after surface.
    window: Window,
}

impl State {
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color.g = position.x / self.size.width as f64;
                self.clear_color.b = position.y / self.size.height as f64;
                self.window.request_redraw();
                true
            }
            _ => false,
        }
    }

    // Creating some of the wgpu types requires async code
    async fn new(window: Window, logger: rwlog::sender::Logger) -> Result<Self, ()> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.map_err(|err| {
            rwlog::rel_err!(&logger, "Failed to create window surface: {err}.");
            ()
        })?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| {
                rwlog::rel_err!(&logger, "Failed to get compatible graphics device.");
                ()
            })?;

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
                ()
            })?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("img/happy_tree.png");
        let diffuse_texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "happy_tree")
            .map_err(|err| {
                rwlog::rel_err!(&logger, "Failed to load texture: {err}.");
                ()
            })?;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let camera = Camera {
            eye: (size.width as f32 / 2.0, size.height as f32 / 2.0, -1.0).into(),
            target: (size.width as f32 / 2.0, size.height as f32 / 2.0, 0.0).into(),
            up: -Vector3::unit_y(),
            left: 0.0,
            right: size.width as f32,
            bottom: size.height as f32,
            top: 0.0,
            znear: 0.0,
            zfar: 100.0,
        };

        let camera_uniform = CameraUniform {
            view_proj: camera.build_view_projection_matrix().into(),
        };

        rwlog::trace!(
            &logger,
            "Frustum limits: width={}; height={}",
            size.width,
            size.height
        );
        rwlog::trace!(
            &logger,
            "View-Projection matrix: {:?}",
            camera.build_view_projection_matrix()
        );
        rwlog::trace!(
            &logger,
            "V0: {:?}",
            camera.build_view_projection_matrix()
                * cgmath::Vector4::<f32> {
                    x: 0.0,
                    y: 0.0,
                    z: 2.0,
                    w: 1.0
                }
        );

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let mesh_uniform_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &MeshUniform::layout_descriptor(),
                label: Some("mesh_bind_group_layout"),
            });

        // Render pipeline #1
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/base.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Base render pipeline layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Base render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex::Textured::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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
                format: texture::Texture::DEPTH_FORMAT,
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

        // Render pipeline #2
        let shader_2 = device.create_shader_module(wgpu::include_wgsl!("shader/general.wgsl"));
        let render_pipeline_layout_2 =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Secondary render pipeline layout."),
                bind_group_layouts: &[&camera_bind_group_layout, &mesh_uniform_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline_2 = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Secondary render pipeline"),
            layout: Some(&render_pipeline_layout_2),
            vertex: wgpu::VertexState {
                module: &shader_2,
                entry_point: "vs_main",
                buffers: &[vertex::Plain::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_2,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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
                format: texture::Texture::DEPTH_FORMAT,
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let vertex_buffer_2 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Secondary vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES2),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer_2 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Secondary index buffer"),
            contents: bytemuck::cast_slice(INDICES2),
            usage: wgpu::BufferUsages::INDEX,
        });

        let button = Button::new(
            &device,
            Point2::<f32> { x: 350.0, y: 250.0 },
            Vector2::<f32> { x: 100.0, y: 100.0 },
            -75.0,
            [0.5, 0.05, 0.05, 1.0],
        );

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            render_pipeline,
            render_pipeline_2,
            active_pipeline: 0,
            vertex_buffer,
            index_buffer,
            vertex_buffer_2,
            index_buffer_2,
            active_mesh: 0,
            diffuse_texture,
            diffuse_bind_group,
            depth_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            button,
            logger,
            size,
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

        {
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

            match self.active_pipeline {
                0 => {
                    render_pass.set_pipeline(&self.render_pipeline);
                    render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
                    let mut indices_num: u32 = 0;

                    match self.active_mesh {
                        0 => {
                            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(
                                self.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            indices_num = INDICES.len() as u32;
                        }
                        1 => {
                            render_pass.set_vertex_buffer(0, self.vertex_buffer_2.slice(..));
                            render_pass.set_index_buffer(
                                self.index_buffer_2.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            indices_num = INDICES2.len() as u32;
                        }
                        _ => {
                            rwlog::rel_err!(&self.logger, "Invalid mesh!");
                        }
                    }

                    render_pass.draw_indexed(0..indices_num as u32, 0, 0..1);
                }
                1 => {
                    render_pass.set_pipeline(&self.render_pipeline_2);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    self.button.draw(&mut render_pass);
                }
                _ => {
                    rwlog::rel_err!(&self.logger, "Invalid render pipeline!");
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    fn update(&mut self) {}

    pub fn window(&self) -> &Window {
        &self.window
    }
}

pub async fn run(logger: rwlog::sender::Logger) {
    env_logger::init(); // Necessary for wgpu error logging.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .unwrap_or_else(|err| {
            rwlog::rel_fatal!(&logger, "Failed to create window: {err}.");
        });

    let mut state = State::new(window, logger.clone())
        .await
        .unwrap_or_else(|_err| {
            rwlog::rel_fatal!(&logger, "Failed to initialise graphics state.");
        });

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } => {
            if window_id == state.window().id() && !state.input(&event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size)
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Space),
                                ..
                            },
                        ..
                    } => {
                        state.active_pipeline = match state.active_pipeline {
                            0 => 1,
                            1 => 0,
                            _ => 0,
                        };
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Key0),
                                ..
                            },
                        ..
                    } => {
                        state.active_mesh = 0;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Key1),
                                ..
                            },
                        ..
                    } => {
                        state.active_mesh = 1;
                    }
                    _ => (),
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();
            match state.render() {
                Ok(_) => (),
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    rwlog::rel_err!(&logger, "Not enough GPU memory!");
                    *control_flow = ControlFlow::Exit;
                }
                Err(e) => {
                    rwlog::warn!(&logger, "{e}");
                }
            };
        }
        Event::MainEventsCleared => {
            state.window().request_redraw();
        }
        _ => (),
    });
}
