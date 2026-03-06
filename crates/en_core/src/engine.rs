use bevy_ecs::world::World;
use std::sync::Arc;
use winit::window::Window;
use glam::{Mat4, Quat, Vec3}; // Додаємо glam для математики

use crate::components::{Sprite, Transform};
use crate::renderer::{Renderer, InstanceRaw};
use crate::time::Time;

pub struct EnEngine {
    pub renderer: Renderer,
    pub world: World,
}

impl EnEngine {
    pub fn new(window: Arc<Window>) -> Self {
        let renderer = pollster::block_on(Renderer::new(window));
        let mut world = World::new();

        world.insert_resource(Time::default());

        world.spawn((
            Transform { x: -200.0, y: 0.0, rotation: 0.0 },
            Sprite { color: [1.0, 0.0, 0.0, 1.0] },
        ));

        world.spawn((
            Transform { x: 200.0, y: 100.0, rotation: 0.0 },
            Sprite { color: [0.0, 1.0, 0.0, 1.0] },
        ));

        Self { renderer, world }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    pub fn update(&mut self) {
        if let Some(mut time) = self.world.get_resource_mut::<Time>() {
            time.update();
        }

        let delta = self.world.get_resource::<Time>().map(|t| t.delta_time).unwrap_or(0.0);

        let mut query = self.world.query::<&mut Transform>();
        for mut transform in query.iter_mut(&mut self.world) {
            transform.rotation += 1.5 * delta; 
        }
    }

    pub fn render(&mut self) -> Result<(), &'static str> {
        let mut instances = Vec::new();
        let mut query = self.world.query::<(&Transform, &Sprite)>();
        
        for (transform, sprite) in query.iter(&self.world) {
            let model_matrix = Mat4::from_scale_rotation_translation(
                Vec3::ONE, 
                Quat::from_rotation_z(transform.rotation), 
                Vec3::new(transform.x, transform.y, 0.0)
            );

            instances.push(InstanceRaw {
                model: model_matrix.to_cols_array_2d(),
                color: sprite.color,
            });
        }

        self.renderer.render(&instances)
    }
}