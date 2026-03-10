use en_macros::en_component;

use crate::Color;

#[en_component]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[en_component]
pub struct Vel {
    pub x: f32,
    pub y: f32,
    pub inertia: f32,
}

#[en_component]
pub struct Render {
    pub s_id: crate::texture_manager::SpriteId,
    pub color: Color, 
    pub layer: f32,
    pub flip_x: bool,
    pub flip_y: bool,
}