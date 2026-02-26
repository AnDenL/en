use macroquad::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AseData {
    pub frames: Vec<AseFrame>,
    pub meta: AseMeta,
}

#[derive(Deserialize, Debug)]
pub struct AseFrame {
    pub frame: AseRect,
    pub duration: i32, 
}

#[derive(Deserialize, Debug, Clone)]
pub struct AseRect {
    pub x: f32, pub y: f32, pub w: f32, pub h: f32,
}

#[derive(Deserialize, Debug)]
pub struct AseMeta {
    #[serde(rename = "frameTags")]
    pub frame_tags: Option<Vec<AseTag>>,
    pub slices: Option<Vec<AseSlice>>,
}

#[derive(Deserialize, Debug)]
pub struct AseTag {
    pub name: String,
    pub from: usize,
    pub to: usize,
}

#[derive(Deserialize, Debug)]
pub struct AseSlice {
    pub name: String,
    pub keys: Vec<AseSliceKey>,
}

#[derive(Deserialize, Debug)]
pub struct AseSliceKey {
    pub frame: usize,
    pub bounds: AseRect,
}