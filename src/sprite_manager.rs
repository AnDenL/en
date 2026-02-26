use std::collections::HashMap;
use macroquad::prelude::*;
use crate::aseprite::*;

pub struct SpriteManager {
    pub textures: Vec<Texture2D>,
    pub sprites: Vec<SpriteData>,
    pub animations: Vec<AnimationData>,

    pub sprite_names: Vec<String>,
    pub name_to_id: HashMap<String, usize>,
}

impl SpriteManager {
    pub fn new() -> Self {
        Self {
            textures: Vec::new(),
            sprites: Vec::new(),
            animations: Vec::new(),
            sprite_names: Vec::new(),
            name_to_id: HashMap::new(),
        }
    }

    fn add_sprite(&mut self, name: String, data: SpriteData) {
        let id = self.sprites.len();
        self.sprites.push(data);
        self.sprite_names.push(name.clone());
        self.name_to_id.insert(name, id);
    }

    pub async fn load_aseprite(&mut self, name: &str, folder: &str) {
        let png_path = format!("{}/{}.png", folder, name);
        let json_path = format!("{}/{}.json", folder, name);

        let texture = load_texture(&png_path).await.expect("Failed to load PNG");
        texture.set_filter(FilterMode::Nearest);
        self.textures.push(texture.clone()); 

        let json_str = macroquad::file::load_string(&json_path).await.expect("Failed to load JSON");
        let ase_data: AseData = serde_json::from_str(&json_str).expect("Failed to parse JSON");
        
        if let Some(slices) = ase_data.meta.slices {
            if slices.is_empty() {
                let f = &ase_data.frames[0].frame;
                self.add_sprite(name.to_string(), SpriteData {
                    texture: texture.clone(),
                    source_rect: Rect::new(f.x, f.y, f.w, f.h),
                });
            } else {
                for slice in slices {
                    if let Some(key) = slice.keys.first() {
                        let frame_rect = &ase_data.frames[key.frame].frame;
                        let bounds = &key.bounds;
                        
                        let atlas_x = frame_rect.x + bounds.x;
                        let atlas_y = frame_rect.y + bounds.y;

                        self.add_sprite(format!("{}_{}", name, slice.name), SpriteData {
                            texture: texture.clone(),
                            source_rect: Rect::new(atlas_x, atlas_y, bounds.w, bounds.h),
                        });
                    }
                }
            }
        } else {
            let f = &ase_data.frames[0].frame;
            self.add_sprite(name.to_string(), SpriteData {
                texture: texture.clone(),
                source_rect: Rect::new(f.x, f.y, f.w, f.h),
            });
        }

        if let Some(tags) = ase_data.meta.frame_tags {
            for tag in tags {
                let mut anim_frames = Vec::new();
                for i in tag.from..=tag.to {
                    let f = &ase_data.frames[i];
                    anim_frames.push(AnimFrame {
                        source_rect: Rect::new(f.frame.x, f.frame.y, f.frame.w, f.frame.h),
                        duration: f.duration as f32 / 1000.0,
                    });
                }

                self.animations.push(AnimationData {
                    texture: texture.clone(),
                    frames: anim_frames,
                });
            }
        }
    }

    pub async fn load_all(&mut self, folder: &str) {
        let index_path = format!("{}/index.json", folder);
        
        if let Ok(json_str) = macroquad::file::load_string(&index_path).await {
            if let Ok(files) = serde_json::from_str::<Vec<String>>(&json_str) {
                let mut sorted_files = files.clone();
                sorted_files.sort();

                for file_name in sorted_files {
                    self.load_aseprite(&file_name, folder).await;
                }
            }
        } else {
            println!("Warning: No index.json found. Did build.rs run?");
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct SpriteId(pub usize); 

#[derive(Clone)]
pub struct SpriteData {
    pub texture: Texture2D,
    pub source_rect: Rect, 
}

#[derive(Clone)]
pub struct AnimFrame {
    pub source_rect: Rect,
    pub duration: f32,
}

#[derive(Clone)]
pub struct AnimationData {
    pub texture: Texture2D,
    pub frames: Vec<AnimFrame>,
}