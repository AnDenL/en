use crate::types::{InstanceData, RenderBatch, Vertex};
use wgpu::util::DeviceExt;

const SPRITE_SHADER: &str = r#"
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
    @location(7) uv_rect: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0, instance.model_matrix_1,
        instance.model_matrix_2, instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.tex_coords = vec2<f32>(
        instance.uv_rect.x + model.tex_coords.x * instance.uv_rect.z,
        instance.uv_rect.y + model.tex_coords.y * instance.uv_rect.w
    );
    out.color = instance.color;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return object_color * in.color;
}
"#;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
    }, // Top-Left
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
    }, // Bottom-Left
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
    }, // Bottom-Right
    Vertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 0.0],
    }, // Top-Right
];
const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

pub struct SpritePass {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    pub texture_bind_group_layout: wgpu::BindGroupLayout,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
}

impl SpritePass {
    pub fn new(device: &wgpu::Device, render_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(SPRITE_SHADER.into()),
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sprite Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&camera_bind_group_layout),
                    Some(&texture_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceData::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_capacity = 10_000;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Global Instance Buffer"),
            size: (instance_capacity * std::mem::size_of::<InstanceData>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            texture_bind_group_layout,
            camera_buffer,
            camera_bind_group,
            instance_capacity,
            instance_buffer,
        }
    }

    pub fn update_camera_buffer(&self, queue: &wgpu::Queue, view_proj: [[f32; 4]; 4]) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[view_proj]));
    }

    pub fn draw<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'a>,
        batches: &'a [RenderBatch<'a>],
        instances: &[InstanceData],
    ) {
        let total_instances = instances.len();
        if total_instances == 0 || batches.is_empty() {
            return;
        }

        if total_instances > self.instance_capacity {
            self.instance_capacity = total_instances.next_power_of_two();
            self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Global Instance Buffer"),
                size: (self.instance_capacity * std::mem::size_of::<InstanceData>())
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        for batch in batches {
            if batch.range.is_empty() {
                continue;
            }

            render_pass.set_bind_group(1, batch.bind_group, &[]);

            // WGPU дозволяє малювати підмасив інстансів завдяки нашому range!
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, batch.range.clone());
        }
    }
}
