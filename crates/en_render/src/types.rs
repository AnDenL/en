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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub uv_rect: [f32; 4],
}

impl InstanceData {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            // Матриця 4x4 займає 4 слоти (location 2, 3, 4, 5)
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 2, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress, shader_location: 3, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress, shader_location: 4, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                // Колір та UV (location 6, 7)
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress, shader_location: 6, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32; 20]>() as wgpu::BufferAddress, shader_location: 7, format: wgpu::VertexFormat::Float32x4 },
            ],
        }
    }
}

pub struct RenderBatch<'a> {
    pub bind_group: &'a wgpu::BindGroup,
    pub instances: &'a [InstanceData],
}