use bevy_ecs::prelude::Resource;
use serde::{Deserialize};
use std::{collections::HashMap, sync::{Mutex, mpsc::{Receiver, Sender, channel}}};

use crate::Rect;

pub enum TextureState {
    Loading,
    Ready {
        bind_group: wgpu::BindGroup,
        width: f32,
        height: f32,
    },
    Error,
}

#[derive(Deserialize)]
pub struct TextureMeta {
    pub texture_path: String,
    pub filter_mode: Option<String>,
    // HashMap, where the key is the slice name (e.g. "idle") and the value is the coordinates
    pub slices: std::collections::HashMap<String, Rect>, 
}

//Sprites

pub const fn hash_string(s: &str) -> u32 {
    let mut hash = 2166136261u32;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u32;
        hash = hash.wrapping_mul(16777619);
        i += 1;
    }
    hash
}

#[derive(Clone)]
pub struct SpriteData {
    pub texture_id: u32,
    pub uv_rect: Rect, // [x, y, w, h]
}

#[derive(Resource)]
pub struct SpriteManager {
    pub textures: HashMap<u32, TextureState>,
    pub sprites: HashMap<u32, SpriteData>,
    
    _texture_rx: Mutex<Receiver<(u32, image::RgbaImage)>>,
    texture_tx: Mutex<Sender<(u32, image::RgbaImage)>>,
}

impl SpriteManager {
    pub fn request_texture(&mut self, name: &str, loader: &crate::assets::AssetLoader) {
        let tex_id = hash_string(name);
        
        if self.textures.contains_key(&tex_id) { return; }
        
        self.textures.insert(tex_id, TextureState::Loading);
        
        let tx = self.texture_tx.lock().unwrap().clone();
        let path = format!("sprites/{}.png", name); // Or read from the meta file
        let loader_clone = (*loader).clone(); 

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(bytes) = loader_clone.load_bytes(&path).await {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let _ = tx.send((tex_id, img.to_rgba8()));
                }
            }
        });

        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            let bytes = pollster::block_on(loader_clone.load_bytes(&path)).unwrap();
            let img = image::load_from_memory(&bytes).unwrap().to_rgba8();
            let _ = tx.send((tex_id, img));
        });
    }

    //pub fn process_loaded_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Read all the images that were loaded in this frame
        //while let Ok((tex_id, rgba_image)) = self.texture_rx.lock().unwrap().try_recv() {
            //let width = rgba_image.width() as f32;
            //let height = rgba_image.height() as f32;
            
            // TODO: Create wgpu::Texture та wgpu::BindGroup with rgba_image
            //let bind_group = create_wgpu_texture(device, queue, &rgba_image); 

            // self.textures.insert(tex_id, TextureState::Ready {
            //     bind_group,
            //     width,
            //     height,
            // });
            
            //println!("[SpriteManager] Texture loaded and sent to GPU!");
        //}
    //}

    pub async fn init_meta(&mut self, loader: &crate::assets::AssetLoader) {
        let index_path = ".en_meta/sprites/index.json";

        match loader.load_json::<Vec<String>>(index_path).await {
            Ok(sprite_names) => {
                for name in sprite_names {
                    self.load_single_meta(&name, loader).await;
                }
                println!("[SpriteManager] Successfully loaded metadata for {} textures.", self.sprites.len());
            }
            Err(e) => {
                eprintln!("[SpriteManager] Warning: No index.json found or error: {}", e);
            }
        }
    }

    async fn load_single_meta(&mut self, name: &str, loader: &crate::assets::AssetLoader) {
        let meta_path = format!(".en_meta/sprites/{}.meta.json", name);
        
        if let Ok(meta) = loader.load_json::<TextureMeta>(&meta_path).await {
            let tex_id = hash_string(name);
            
            // If there are slices in the metadata
            if !meta.slices.is_empty() {
                for (slice_name, rect) in meta.slices {
                    // Form the name of the sprite, for example "player_idle"
                    let sprite_name = format!("{}_{}", name, slice_name);
                    let sprite_id = hash_string(&sprite_name);
                    
                    self.sprites.insert(sprite_id, SpriteData {
                        texture_id: tex_id,
                        uv_rect: rect,
                    });
                }
            } else {
                // If there is no slice, then the whole picture is one sprite
                // (For now, we are setting conditional zeros, they will need to be replaced with real sizes when creating the texture,
                // or the size must be specified in the metadata)
                let sprite_id = hash_string(name);
                self.sprites.insert(sprite_id, SpriteData {
                    texture_id: tex_id,
                    uv_rect: Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }, // UV coordinates for the entire texture
                });
            }
        } else {
            eprintln!("Failed to load meta for {}", name);
        }
    }
}

impl Default for SpriteManager {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            textures: HashMap::new(),
            sprites: HashMap::new(),
            _texture_rx: Mutex::new(rx),
            texture_tx: Mutex::new(tx),
        }
    }
}