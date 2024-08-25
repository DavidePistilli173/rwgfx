//! Textures.

use anyhow::*;
use cgmath::Vector2;
use image::GenericImageView;

pub use wgpu::Origin3d;
pub use wgpu::TextureFormat;

/// Invalid texture ID.
pub const ID_INVALID: u64 = 0;
/// Empty texture ID.
pub const ID_EMPTY: u64 = 1;
/// Hamburger menu icon ID.
pub const ID_HAMBURGER: u64 = 2;

/// Get the appropriate data layout for a given texture format and size.
fn image_data_layout(format: TextureFormat, extent: wgpu::Extent3d) -> wgpu::ImageDataLayout {
    let (block_x, block_y) = format.block_dimensions();
    let bytes_per_block = format.block_size(None).unwrap_or(0);
    let bytes_per_row = Some(bytes_per_block * extent.width / block_x);
    let rows_per_image = Some(extent.height / block_y);

    wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row,
        rows_per_image,
    }
}

/// Structure containing texture information.
#[derive(Debug)]
pub struct Texture {
    /// Actual texture.
    pub texture: wgpu::Texture,
    /// Texture view.
    pub view: wgpu::TextureView,
    /// Texture sampler.
    pub sampler: wgpu::Sampler,
    /// Bind group.
    pub bind_group: wgpu::BindGroup,
    /// Texture size.
    pub size: wgpu::Extent3d,
    /// Data format of the texture.
    pub format: TextureFormat,
}

impl Texture {
    /// Format of the depth textures.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Get the bind group layout for a texture.
    pub fn bind_group_layout(
        device: &wgpu::Device,
        format: TextureFormat,
    ) -> wgpu::BindGroupLayout {
        let sample_type = format
            .sample_type(None)
            .unwrap_or(wgpu::TextureSampleType::default());

        let sampler_binding_type = match sample_type {
            wgpu::TextureSampleType::Depth => wgpu::SamplerBindingType::Comparison,
            wgpu::TextureSampleType::Float { filterable: true } => {
                wgpu::SamplerBindingType::Filtering
            }
            wgpu::TextureSampleType::Float { filterable: false } => {
                wgpu::SamplerBindingType::NonFiltering
            }
            wgpu::TextureSampleType::Sint => wgpu::SamplerBindingType::Filtering,
            wgpu::TextureSampleType::Uint => wgpu::SamplerBindingType::Filtering,
        };

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(sampler_binding_type),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
    }

    /// Create a depth texture.
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("depth_bind_group"),
        });

        Self {
            texture,
            view,
            sampler,
            bind_group,
            size,
            format: Self::DEPTH_FORMAT,
        }
    }

    /// Utility function for creating a texture sampler.
    fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    /// Create a texture from a slice of raw bytes.
    pub fn from_bytes(
        ctx: &rwcompute::Context,
        bytes: &[u8],
        size: Vector2<u32>,
        format: TextureFormat,
        label: &str,
    ) -> Result<Self> {
        let size = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };

        let texture = ctx.device().create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        ctx.queue().write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            image_data_layout(format, size),
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = Self::create_sampler(ctx.device());

        let bind_group_layout = Texture::bind_group_layout(ctx.device(), format);
        let bind_group = ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some(label),
        });

        Ok(Self {
            texture,
            view,
            sampler,
            bind_group,
            size,
            format,
        })
    }

    /// Create a texture from an image.
    pub fn from_image(
        ctx: &rwcompute::Context,
        img: image::DynamicImage,
        label: &str,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = ctx.device().create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        ctx.queue().write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            image_data_layout(wgpu::TextureFormat::Rgba8UnormSrgb, size),
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = Self::create_sampler(ctx.device());

        let bind_group_layout =
            Texture::bind_group_layout(ctx.device(), wgpu::TextureFormat::Rgba8UnormSrgb);
        let bind_group = ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        Ok(Self {
            texture,
            view,
            sampler,
            bind_group,
            size,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
        })
    }

    /// Write new data to a texture.
    /// The data must be in the same format as the texture.
    pub fn write_data(
        &self,
        queue: &wgpu::Queue,
        bytes: &[u8],
        size: Vector2<u32>,
        offset: Origin3d,
    ) {
        let size = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 0,
        };

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: offset,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            image_data_layout(self.format, size),
            size,
        );
    }
}
