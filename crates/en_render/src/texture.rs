use crate::config::FilterMode;

pub struct GpuTexture;

impl GpuTexture {
    /// Створює текстуру та BindGroup з сирих RGBA байтів.
    /// Це саме те, що буде викликати `TextureManager` з `en_core`.
    pub fn create_texture_bind_group(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        rgba_data: &[u8],
        width: u32,
        height: u32,
        filter: FilterMode,
    ) -> wgpu::BindGroup {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("EnEngine Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let (wgpu_filter, mipmap_filter) = match filter {
            FilterMode::Nearest => (wgpu::FilterMode::Nearest, wgpu::MipmapFilterMode::Nearest),
            FilterMode::Linear => (wgpu::FilterMode::Linear, wgpu::MipmapFilterMode::Linear),
        };

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu_filter,
            min_filter: wgpu_filter,
            mipmap_filter: mipmap_filter,
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
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
            label: Some("texture_bind_group"),
        })
    }

    /// Створює дефолтну білу текстуру 1x1 піксель.
    /// Якщо ми малюємо батч без текстури, ми використовуємо її (тоді множення на колір дасть просто колір).
    pub fn create_default_white_bind_group(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        // Один білий непрозорий піксель (R:255, G:255, B:255, A:255)
        let white_pixel: [u8; 4] = [255, 255, 255, 255];
        Self::create_texture_bind_group(
            device,
            queue,
            layout,
            &white_pixel,
            1,
            1,
            FilterMode::Nearest,
        )
    }
}
