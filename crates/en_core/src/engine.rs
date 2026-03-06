use bevy_ecs::world::World;
use std::sync::Arc;
use winit::window::Window;

use crate::components::{Sprite, Transform};
use crate::renderer::Renderer;

pub struct EnEngine {
    pub renderer: Renderer,
    pub world: World,
}

impl EnEngine {
    pub fn new(window: Arc<Window>) -> Self {
        let renderer = pollster::block_on(Renderer::new(window));

        let mut world = World::new();

        world.spawn((
            Transform { x: 0.0, y: 0.0, rotation: 0.0 },
            Sprite { color: [1.0, 0.0, 0.0, 1.0] },
        ));

        Self { renderer, world }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    pub fn update(&mut self) {
        
    }

    pub fn render(&mut self) -> Result<(), &'static str> {
        self.renderer.render()
    }
}