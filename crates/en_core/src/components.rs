use crate::Color;
use crate::prelude::{ReflectComponent, ReflectDefault};
use en_macros::en_component;

#[en_component]
pub struct Name(pub String);

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

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
pub struct SpriteRenderer {
    //pub s_id: SpriteId,
    pub color: Color,
    pub layer: f32,
    pub flip_x: bool,
    pub flip_y: bool,
}
