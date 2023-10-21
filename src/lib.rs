use wgpu::util::DeviceExt;
use winit::{
    event::{self, *},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    colour: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        colour: [0.5, 0.0, 0.0],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        colour: [0.5, 0.5, 0.0],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        colour: [0.0, 0.5, 0.0],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        colour: [0.0, 0.5, 0.5],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        colour: [0.0, 0.0, 0.5],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

const VERTICES2: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        colour: [1.0, 0.0, 0.0],
    }, // A
    Vertex {
        position: [0.2, 0.2, 0.0],
        colour: [1.0, 1.0, 0.0],
    }, // B
    Vertex {
        position: [-0.2, 0.2, 0.0],
        colour: [1.0, 1.0, 0.0],
    }, // C
    Vertex {
        position: [0.5, 0.2, 0.0],
        colour: [1.0, 0.0, 0.0],
    }, // D
    Vertex {
        position: [0.3, -0.2, 0.0],
        colour: [1.0, 1.0, 0.0],
    }, // E
    Vertex {
        position: [0.4, -0.5, 0.0],
        colour: [1.0, 0.0, 0.0],
    }, // F
    Vertex {
        position: [0.0, -0.2, 0.0],
        colour: [1.0, 1.0, 0.0],
    }, // G
    Vertex {
        position: [-0.4, -0.5, 0.0],
        colour: [1.0, 0.0, 0.0],
    }, // H
    Vertex {
        position: [-0.3, -0.2, 0.0],
        colour: [1.0, 1.0, 0.0],
    }, // I
    Vertex {
        position: [-0.5, 0.2, 0.0],
        colour: [1.0, 0.0, 0.0],
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
        let diffuse_image = image::load_from_memory(diffuse_bytes);

        // Render pipeline #1
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/base.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Base render pipeline layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Base render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Render pipeline #2
        let shader_2 = device.create_shader_module(wgpu::include_wgsl!("shader/base2.wgsl"));
        let render_pipeline_layout_2 =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Secondary render pipeline layout."),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline_2 = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Secondary render pipeline"),
            layout: Some(&render_pipeline_layout_2),
            vertex: wgpu::VertexState {
                module: &shader_2,
                entry_point: "vs_main",
                buffers: &[],
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
            depth_stencil: None,
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
                depth_stencil_attachment: None,
            });

            match self.active_pipeline {
                0 => {
                    render_pass.set_pipeline(&self.render_pipeline);
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
                    render_pass.draw(0..3 as u32, 0..1);
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
