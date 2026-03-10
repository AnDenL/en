use std::ops::Deref;
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self { Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } }
}

impl Deref for Color {
    type Target = [f32; 4];

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}