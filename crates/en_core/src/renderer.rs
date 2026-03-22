use std::sync::Arc;
use bevy_ecs::world::World;
use glam::{Mat4, Quat, Vec3};
use winit::window::Window;

// Імпортуємо типи з нової бібліотеки
use en_render::{RenderSettings, InstanceData, RenderBatch};
use crate::{
    camera::{Camera2D, CameraUniform}, 
    engine::EditorSelected, 
    prelude::{SpriteRenderer, Transform},
    texture_manager::{TextureManager, TextureState}
};

// Структура для збереження даних кадру (володіє векторами, щоб RenderBatch міг на них посилатися)
pub struct FrameData {
    pub default_instances: Vec<InstanceData>,
    pub texture_batches: std::collections::HashMap<u32, Vec<InstanceData>>,
}

// Обгортка для рушія, яка тримає рендер та активну камеру
pub struct Renderer {
    pub inner: en_render::Renderer,
    pub camera: Camera2D,
    pub window: Option<Arc<Window>>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let settings = RenderSettings::default(); // Можна брати з ProjectConfig
        let inner = en_render::Renderer::new(window.clone(), settings).await.unwrap();
        
        Self { 
            inner, 
            camera: Camera2D::default(), 
            window: Some(window) 
        }
    }

    pub fn new_for_editor(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, format: wgpu::TextureFormat) -> Self {
        let settings = RenderSettings::default();
        let inner = en_render::Renderer::new_headless(device, queue, format, settings);
        
        Self { 
            inner, 
            camera: Camera2D::default(), 
            window: None 
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.inner.resize(new_size.width, new_size.height);
        self.camera.update_aspect_ratio(new_size.width as f32, new_size.height as f32);
    }

    fn update_camera_to_gpu(&mut self) {
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(&self.camera);
        // Передаємо матрицю в en_render
        self.inner.update_camera(uniform.view_proj);
    }

    // Головний метод малювання для гри
    pub fn render(&mut self, world: &mut World) -> Result<(), &'static str> {
        self.update_camera_to_gpu();
        let frame_data = build_frame_data(world);
        let batches = self.create_batches(&frame_data, world);
        
        self.inner.render(&batches).map_err(|_| "Surface error")
    }

    // Головний метод малювання для редактора
    pub fn render_editor_view(&mut self, world: &mut World, view: &wgpu::TextureView) {
        self.update_camera_to_gpu();
        let frame_data = build_frame_data(world);
        let batches = self.create_batches(&frame_data, world);
        
        self.inner.render_to_view(&batches, view);
    }

    // Збираємо посилання на BindGroups разом зі згенерованими InstanceData
    fn create_batches<'a>(&'a self, frame_data: &'a FrameData, world: &'a World) -> Vec<RenderBatch<'a>> {
        let texture_manager = world.get_resource::<TextureManager>().unwrap();
        let mut batches = Vec::new();

        if !frame_data.default_instances.is_empty() {
            batches.push(RenderBatch {
                bind_group: &self.inner.default_white_bind_group,
                instances: &frame_data.default_instances,
            });
        }
        
        for (id, instances) in &frame_data.texture_batches {
            if let Some(TextureState::Loaded { bind_group, .. }) = texture_manager.get_texture(crate::texture_manager::TextureId(*id)) {
                batches.push(RenderBatch {
                    bind_group,
                    instances,
                });
            }
        }

        batches
    }
}

// Функція збору ECS даних у FrameData
fn build_frame_data(world: &World) -> FrameData {
    let mut default_instances = Vec::new();
    let mut texture_batches: std::collections::HashMap<u32, Vec<InstanceData>> = std::collections::HashMap::new();

    let mut query = world.query::<(
        &Transform, 
        &SpriteRenderer,
        Option<&EditorSelected>
    )>();

    let texture_manager = world.get_resource::<TextureManager>().unwrap();
    
    for (transform, render, selected) in query.iter(world) {
        let mut color = render.color.to_array(); // Припускаємо, що Color має to_array()
        let mut uv_rect = [0.0, 0.0, 1.0, 1.0];
        let mut tex_id = None;
        let mut scale = Vec3::ONE;

        if let Some(s) = texture_manager.sprites.get(&render.s_id) {
            tex_id = Some(s.texture_id.0);
            uv_rect = [s.uv_rect.x, s.uv_rect.y, s.uv_rect.w, s.uv_rect.h];
            
            // Якщо є параметр PPU, враховуємо його
            scale.x = s.pixel_rect.w / s.ppu;
            scale.y = s.pixel_rect.h / s.ppu;
        }

        let position = Vec3::new(transform.x, transform.y, render.layer); // Якщо є layer, можна юзати в Z
        let rotation = Quat::from_rotation_z(transform.rotation);
        let model_matrix = Mat4::from_scale_rotation_translation(scale, rotation, position);

        let instance = InstanceData { 
            model_matrix: model_matrix.to_cols_array_2d(), 
            color,
            uv_rect,
        };

        if let Some(id) = tex_id {
            texture_batches.entry(id).or_default().push(instance);
        } else {
            default_instances.push(instance);
        }

        // Логіка обводки (для редактора)
        if selected.is_some() {
            let outline_matrix = Mat4::from_scale_rotation_translation(
                scale * 1.05, rotation, position
            );
            let outline_instance = InstanceData { 
                model_matrix: outline_matrix.to_cols_array_2d(), 
                color: [1.0, 0.8, 0.0, 1.0], // Жовта обводка
                uv_rect,
            };
            if let Some(id) = tex_id {
                texture_batches.entry(id).or_default().push(outline_instance);
            } else {
                default_instances.push(outline_instance);
            }
        }
    }

    FrameData {
        default_instances,
        texture_batches,
    }
}