use en_macros::en_component;

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
pub struct Sprite {
    #[default([1.0, 1.0, 1.0, 1.0])]
    pub color: [f32; 4],
}