use en_macros::en_component;

#[en_component]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}
impl Default for Transform {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, rotation: 0.0 }
    }
}

#[en_component]
pub struct Sprite {
    pub color: [f32; 4],
}
impl Default for Sprite {
    fn default() -> Self {
        Self { color: [0.0,0.0,0.0,0.0] }
    }
}