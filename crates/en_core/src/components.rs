use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Component)]
pub struct Sprite {
    pub color: [f32; 4],
}