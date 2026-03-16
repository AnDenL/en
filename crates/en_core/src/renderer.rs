use std::sync::Arc;
use bevy_ecs::world::{World};
use glam::{Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;
use winit::window::Window;
use crate::{camera::{Camera2D, CameraUniform}, engine::EditorSelected, prelude::{SpriteRenderer, Transform}};

const SHADER_SOURCE: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) color: vec4<f32>,
    @location(7) uv_rect: vec4<f32>, // x, y, w, h
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    
    // Перетворення UV координат за допомогою rect (ідеально для спрайт-листів)
    out.tex_coords = vec2<f32>(
        instance.uv_rect.x + model.tex_coords.x * instance.uv_rect.z,
        instance.uv_rect.y + model.tex_coords.y * instance.uv_rect.w
    );
    
    out.color = instance.color;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return object_color * in.color;
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2], 
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
                wgpu::VertexAttribute { offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5,  0.5, 0.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] }, 
    Vertex { position: [ 0.5,  0.5, 0.0], tex_coords: [1.0, 0.0] },
];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub uv_rect: [f32; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 2, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress, shader_location: 3, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress, shader_location: 4, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress, shader_location: 6, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 20]>() as wgpu::BufferAddress, shader_location: 7, format: wgpu::VertexFormat::Float32x4 },
            ],
        }
    }
}

pub struct RenderBatch<'a> {
    pub bind_group: &'a wgpu::BindGroup,
    pub instances: Vec<InstanceRaw>,
}

pub struct Renderer {
    pub surface: Option<wgpu::Surface<'static>>, 
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,  
    pub config: Option<wgpu::SurfaceConfiguration>,
    pub window: Option<Arc<Window>>,
    pub render_format: wgpu::TextureFormat, 
    
    pub render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    pub camera_uniform: CameraUniform,
    pub camera: Camera2D,

    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub default_texture_bind_group: wgpu::BindGroup,
}

impl Renderer {
    fn build_shared_resources(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> (
        wgpu::RenderPipeline, wgpu::Buffer, wgpu::Buffer, u32, wgpu::Buffer, wgpu::BindGroup, CameraUniform, Camera2D, wgpu::BindGroupLayout, wgpu::BindGroup
    ) {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Main Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let mut camera_uniform = CameraUniform::new();
        let camera = Camera2D::default();
        camera_uniform.update_view_proj(&camera);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
            label: Some("camera_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() }],
            label: Some("camera_bind_group"),
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
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

        let default_texture_view = Self::create_blank_texture(device, queue);
        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let default_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&default_texture_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&default_sampler) },
            ],
            label: Some("default_texture_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
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

        (render_pipeline, vertex_buffer, index_buffer, INDICES.len() as u32, camera_buffer, camera_bind_group, camera_uniform, camera, texture_bind_group_layout, default_texture_bind_group)
    }

    fn create_blank_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
        let size = wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("White Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            size,
        );
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..Default::default() }).await.unwrap();
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await.unwrap();
        
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, format, width: size.width, height: size.height,
            present_mode: wgpu::PresentMode::Fifo, alpha_mode: caps.alpha_modes[0], view_formats: vec![], desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let (render_pipeline, vertex_buffer, index_buffer, num_indices, camera_buffer, camera_bind_group, camera_uniform, camera, texture_bind_group_layout, default_texture_bind_group) = 
            Self::build_shared_resources(&device, &queue, format);

        Self {
            surface: Some(surface), device: Arc::new(device), queue: Arc::new(queue), config: Some(config), window: Some(window), render_format: format,
            render_pipeline, vertex_buffer, index_buffer, num_indices, camera_buffer, camera_bind_group, camera_uniform, camera,
            texture_bind_group_layout, default_texture_bind_group
        }
    }

    pub fn new_for_editor(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, format: wgpu::TextureFormat) -> Self {
        let (render_pipeline, vertex_buffer, index_buffer, num_indices, camera_buffer, camera_bind_group, camera_uniform, camera, texture_bind_group_layout, default_texture_bind_group) = 
            Self::build_shared_resources(&device, &queue, format);

        Self {
            surface: None, device, queue, config: None, window: None, render_format: format,
            render_pipeline, vertex_buffer, index_buffer, num_indices, camera_buffer, camera_bind_group, camera_uniform, camera,
            texture_bind_group_layout, default_texture_bind_group
        }
    }

    pub fn render_to_view(&self, batches: &[RenderBatch], view: &wgpu::TextureView) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }), store: wgpu::StoreOp::Store },
                    depth_slice: None,
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]); 
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            for batch in batches {
                if batch.instances.is_empty() { continue; }
                
                let instance_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&batch.instances),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                render_pass.set_bind_group(1, batch.bind_group, &[]); 
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..)); 
                render_pass.draw_indexed(0..self.num_indices, 0, 0..batch.instances.len() as u32);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn render(&self, batches: &[RenderBatch]) -> Result<(), &'static str> {
        let surface = self.surface.as_ref().ok_or("No surface")?;
        let output = surface.get_current_texture().map_err(|_| "Surface error")?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.render_to_view(batches, &view);
        output.present();
        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let (Some(config), Some(surface)) = (&mut self.config, &self.surface) {
            config.width = new_size.width; config.height = new_size.height;
            surface.configure(&self.device, config);
        }
        self.camera.update_aspect_ratio(new_size.width as f32, new_size.height as f32);
        self.update_camera_buffer();
    }

    pub fn update_camera_buffer(&mut self) {
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }
}

pub fn build_render_batches<'a>(world: &'a mut World, renderer: &'a Renderer) -> Vec<RenderBatch<'a>> {
    let mut default_instances = Vec::new();
    let mut texture_batches: std::collections::HashMap<u32, Vec<InstanceRaw>> = std::collections::HashMap::new();

    let mut query = world.query::<(
        &Transform, 
        &SpriteRenderer,
        Option<&EditorSelected>
    )>();

    let sprite_manager = world.get_resource::<crate::texture_manager::SpriteManager>().unwrap();
    
    for (transform, render, selected) in query.iter(&world) {
        let mut color = [1.0, 1.0, 1.0, 1.0];
        let mut uv_rect = [0.0, 0.0, 1.0, 1.0];
        let mut tex_id = None;
        let mut scale = Vec3::ONE;

        if let Some(s) = sprite_manager.sprites.get(&render.s_id.0) {
            tex_id = Some(s.texture_id);
            if let Some(t) = sprite_manager.textures.get(&s.texture_id){ 
                uv_rect = [s.uv_rect.x, s.uv_rect.y, s.uv_rect.w, s.uv_rect.h];
                
                scale.x = s.pixel_rect.w / crate::texture_manager::PPU;
                scale.y = s.pixel_rect.h / crate::texture_manager::PPU;
            }
            color = render.color.to_array();
        }

        let position = Vec3::new(transform.x, transform.y, 0.0);
        let rotation = Quat::from_rotation_z(transform.rotation);
        let model_matrix = Mat4::from_scale_rotation_translation(scale, rotation, position);

        let instance = InstanceRaw { 
            model: model_matrix.to_cols_array_2d(), 
            color,
            uv_rect,
        };

        if let Some(id) = tex_id {
            texture_batches.entry(id).or_default().push(instance);
        } else {
            default_instances.push(instance);
        }

        if selected.is_some() {
            let outline_matrix = Mat4::from_scale_rotation_translation(
                scale * 1.05, rotation, position
            );
            let outline_instance = InstanceRaw { 
                model: outline_matrix.to_cols_array_2d(), 
                color: [1.0, 0.8, 0.0, 1.0],
                uv_rect,
            };
            if let Some(id) = tex_id {
                texture_batches.entry(id).or_default().push(outline_instance);
            } else {
                default_instances.push(outline_instance);
            }
        }
    }

    let mut batches = Vec::new();
    if !default_instances.is_empty() {
        batches.push(RenderBatch {
            bind_group: &renderer.default_texture_bind_group,
            instances: default_instances,
        });
    }
    
    for (id, instances) in texture_batches {
        if let Some(t) = sprite_manager.textures.get(&id) {
            let bind_group = &t.bind_group;
            batches.push(RenderBatch {
                bind_group,
                instances,
            });
        }
    }

    batches
}