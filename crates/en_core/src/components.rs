use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, Component)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Component)]
pub struct Sprite {
    pub color: [f32; 4],
}