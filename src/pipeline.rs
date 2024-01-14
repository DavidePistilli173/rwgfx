//! Pipeline utility code and deafult pipeline IDs.

use crate::texture::Texture;

/// Invalid pipeline ID.
pub const ID_INVALID: u64 = 0;
/// ID of the general pipeline.
pub const ID_GENERAL: u64 = 1;

/// Get the default depth stencil state.
pub fn default_depth_stencil_state() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: Texture::DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }
}

/// Get the default multisample state.
pub const fn default_multisample_state() -> wgpu::MultisampleState {
    wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
    }
}

/// Macro for creating a render pipeline with default options.
#[macro_export]
macro_rules! create_default_render_pipeline {
    ($device:expr, $surface_config:expr, $shader_name:expr, $shader_obj:expr, $bind_group_layouts:expr, $vertex_buffer_layouts:expr) => {
        $device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{} render pipeline", $shader_name)),
            layout: Some(
                &$device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{} render pipeline layout.", $shader_name)),
                    bind_group_layouts: $bind_group_layouts,
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &$shader_obj,
                entry_point: "vs_main",
                buffers: $vertex_buffer_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &$shader_obj,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: $surface_config.format,
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
            depth_stencil: Some(crate::pipeline::default_depth_stencil_state()),
            multisample: crate::pipeline::default_multisample_state(),
            multiview: None,
        })
    };
}
