use macroquad::{prelude::*};

crate::define_all_components! {
    Pos { x: f32, y: f32 },

    Vel { x: f32, y: f32, d: f32 = 10.0 },

    Render { 
        s_id: crate::sprite_manager::SpriteId = crate::sprite_manager::SpriteId(0),
        w: f32 = crate::PPU, 
        h: f32 = crate::PPU, 
        color: [f32; 4] = [1.0, 1.0, 1.0, 1.0],
        layer: f32 = 0.0,
        flip_x: bool = false,
        flip_y: bool = false,

        #[serde(skip)]
        cached_sprite: Option<crate::sprite_manager::SpriteData> = None,
    },

    CameraAnchor { zoom: f32 = 1.0, smoothness: f32 = 1.0 },

    Player { speed: f32 = 50.0 },

    Collider { w: f32 = crate::PPU, h: f32 = crate::PPU, is_static: bool = true },
}

impl Render {
    pub fn r(&self) -> f32 { self.color[0] }
    pub fn g(&self) -> f32 { self.color[1] }
    pub fn b(&self) -> f32 { self.color[2] }
    pub fn a(&self) -> f32 { self.color[3] }
}