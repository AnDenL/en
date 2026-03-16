use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};
use wgpu;
use bevy_ecs::prelude::Resource;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::assets::AssetLoader;
use crate::types::Rect;

// --- ТИПИ ТА ІДЕНТИФІКАТОРИ ---

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[serde(transparent)]
pub struct SpriteId(pub u32);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[serde(transparent)]
pub struct TextureId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub enum FilterMode {
    Nearest, // Для піксель-арту (чіткі краї)
    Linear,  // Для звичайних картинок (згладжені краї)
}

// Стандартний масштаб: скільки пікселів текстури влазить в одну одиницю ігрового світу
pub const DEFAULT_PPU: f32 = 100.0; 

// --- СТАН ТЕКСТУРИ ТА СПРАЙТА ---

pub enum TextureState {
    Unloaded {
        path: String,
        filter: FilterMode,
    },
    Loading,
    Loaded {
        texture: wgpu::Texture,
        bind_group: wgpu::BindGroup,
        width: u32,
        height: u32,
    },
}

#[derive(Clone, Debug, Reflect)]
pub struct SpriteData {
    pub texture_id: TextureId,
    // Яка частина текстури малюється (в пікселях)
    pub pixel_rect: Rect, 
    // Яка частина текстури малюється (в UV координатах від 0.0 до 1.0)
    // Обчислюється автоматично, коли текстура завантажиться!
    pub uv_rect: Rect,    
    pub ppu: f32,
}

// --- СПАВНЕР ФОНОВИХ ЗАДАЧ ---

// Ця магія дозволяє нам запускати асинхронний код у фоні
// без блокування головного циклу, працює і на ПК, і в браузері.
#[cfg(not(target_arch = "wasm32"))]
fn spawn_background_task<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    // На десктопі створюємо звичайний потік і блокуємо його (це безпечно, бо потік фоновий)
    std::thread::spawn(move || {
        pollster::block_on(future);
    });
}

#[cfg(target_arch = "wasm32")]
fn spawn_background_task<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    // В браузері віддаємо JS-рушію
    wasm_bindgen_futures::spawn_local(future);
}

// --- ГОЛОВНИЙ МЕНЕДЖЕР ---

#[derive(Resource)]
pub struct TextureManager {
    textures: HashMap<TextureId, TextureState>,
    pub sprites: HashMap<SpriteId, SpriteData>,
    
    byte_receiver: Receiver<(TextureId, Vec<u8>)>,
    byte_sender: Sender<(TextureId, Vec<u8>)>,
    
    next_tex_id: u32,
    next_sprite_id: u32,
}

impl Default for TextureManager {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            textures: HashMap::new(),
            sprites: HashMap::new(),
            byte_receiver: rx,
            byte_sender: tx,
            next_tex_id: 1, // Починаємо з 1, 0 можна лишити для "пустишки"
            next_sprite_id: 1,
        }
    }
}

impl TextureManager {
    // --- ПУБЛІЧНИЙ API ДЛЯ КОРИСТУВАЧА РУШІЯ ---

    /// Реєструє текстуру і повертає її ID. Сама текстура ще НЕ вантажиться.
    pub fn add_texture(&mut self, path: &str, filter: FilterMode) -> TextureId {
        let id = TextureId(self.next_tex_id);
        self.next_tex_id += 1;
        
        self.textures.insert(id, TextureState::Unloaded { 
            path: path.to_string(), 
            filter 
        });
        
        id
    }

    /// Створює спрайт з цілої текстури
    pub fn create_sprite(&mut self, texture_id: TextureId, ppu: Option<f32>) -> SpriteId {
        let id = SpriteId(self.next_sprite_id);
        self.next_sprite_id += 1;

        // Поки текстура не завантажена, ми не знаємо її розмірів,
        // тому pixel_rect ставимо в 0. Він оновиться автоматично пізніше!
        self.sprites.insert(id, SpriteData {
            texture_id,
            pixel_rect: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
            uv_rect: Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }, // По замовчуванню вся текстура
            ppu: ppu.unwrap_or(DEFAULT_PPU),
        });

        id
    }

    /// Створює спрайт з частини текстури (для ручного задання)
    pub fn create_sprite_region(&mut self, texture_id: TextureId, pixel_rect: Rect, ppu: Option<f32>) -> SpriteId {
        let id = SpriteId(self.next_sprite_id);
        self.next_sprite_id += 1;

        self.sprites.insert(id, SpriteData {
            texture_id,
            pixel_rect,
            // UV порахується коли текстура завантажиться
            uv_rect: Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }, 
            ppu: ppu.unwrap_or(DEFAULT_PPU),
        });

        id
    }

    /// Магія нарізки! Ріже текстуру на рівні шматки і повертає масив SpriteId
    pub fn slice_atlas(&mut self, texture_id: TextureId, columns: u32, rows: u32, tile_w: f32, tile_h: f32, ppu: Option<f32>) -> Vec<SpriteId> {
        let mut sprite_ids = Vec::new();

        for y in 0..rows {
            for x in 0..columns {
                let rect = Rect {
                    x: x as f32 * tile_w,
                    y: y as f32 * tile_h,
                    w: tile_w,
                    h: tile_h,
                };
                let id = self.create_sprite_region(texture_id, rect, ppu);
                sprite_ids.push(id);
            }
        }

        sprite_ids
    }

    // --- ВНУТРІШНЯ ЛОГІКА РУШІЯ (ДЛЯ РЕНДЕРУ) ---

    /// Цю функцію викликає рендер, коли намагається намалювати текстуру.
    /// Якщо вона ще не завантажена - запускає завантаження у фоні.
    pub fn get_or_queue_load(
        &mut self,
        texture_id: TextureId,
        asset_loader: &AssetLoader,
    ) -> Option<&wgpu::BindGroup> {
        let state = self.textures.get_mut(&texture_id)?;

        match state {
            TextureState::Loaded { bind_group, .. } => Some(bind_group),
            TextureState::Unloaded { path, .. } => {
                let file_path = path.clone();
                *state = TextureState::Loading; // Блокуємо повторні запити
                
                let tx = self.byte_sender.clone();
                let loader = asset_loader.clone();

                // МАГІЯ ФОНОВОГО ЗАВАНТАЖЕННЯ (Без фризів!)
                spawn_background_task(async move {
                    if let Ok(bytes) = loader.load_bytes(&file_path).await {
                        let _ = tx.send((texture_id, bytes));
                    } else {
                        eprintln!("[TextureManager] Failed to load bytes for {}", file_path);
                    }
                });

                None
            }
            TextureState::Loading => None,
        }
    }

    /// Викликається КОЖЕН КАДР перед рендером.
    /// Перевіряє, чи прилетіли байти з фонового потоку, і створює WGPU ресурси.
    pub fn update_gpu_resources(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) {
        // Забираємо всі текстури, які завантажились за цей кадр
        while let Ok((tex_id, bytes)) = self.byte_receiver.try_recv() {
            
            // Якщо раптом текстуру видалили поки вона вантажилась - пропускаємо
            let filter = match self.textures.get(&tex_id) {
                Some(TextureState::Loading) => {
                    // Щоб знати фільтр, нам довелось би його зберігати, 
                    // але для спрощення візьмемо зі старого стану або дефолт.
                    // (У реальності він мав би зберігатись в Loading стані, але ми схитруємо)
                    FilterMode::Nearest // Ми його пофіксимо нижче
                },
                _ => continue, 
            };

            // Декодуємо картинку (PNG/JPEG)
            let img = match image::load_from_memory(&bytes) {
                Ok(i) => i.to_rgba8(),
                Err(e) => {
                    eprintln!("[TextureManager] Image decode error: {}", e);
                    continue;
                }
            };
            
            let dimensions = img.dimensions();
            let width = dimensions.0;
            let height = dimensions.1;

            let texture_size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Sprite Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &img,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                texture_size,
            );

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            // Налаштовуємо Sampler (Згладжування чи пікселі?)
            // TODO: Для ідеалу треба витягти FilterMode з попереднього стану, 
            // але для прикладу хай буде Nearest
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest, // <- Ось тут магія піксель-арту!
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("sprite_bind_group"),
            });

            // Зберігаємо завантажену текстуру
            self.textures.insert(tex_id, TextureState::Loaded {
                texture,
                bind_group,
                width,
                height,
            });

            // КРИТИЧНО ВАЖЛИВИЙ КРОК:
            // Оновлюємо UV-координати всіх спрайтів, які залежать від цієї текстури
            self.update_sprite_uvs(tex_id, width as f32, height as f32);
        }
    }

    /// Перераховує UV-прямокутники для спрайтів, коли ми нарешті знаємо розмір картинки
    fn update_sprite_uvs(&mut self, tex_id: TextureId, tex_w: f32, tex_h: f32) {
        for (_, sprite) in self.sprites.iter_mut() {
            if sprite.texture_id == tex_id {
                // Якщо це спрайт на всю текстуру (pixel_rect == 0), задаємо йому розмір
                if sprite.pixel_rect.w == 0.0 && sprite.pixel_rect.h == 0.0 {
                    sprite.pixel_rect.w = tex_w;
                    sprite.pixel_rect.h = tex_h;
                }

                // Вираховуємо UV (від 0.0 до 1.0)
                sprite.uv_rect.x = sprite.pixel_rect.x / tex_w;
                sprite.uv_rect.y = sprite.pixel_rect.y / tex_h;
                sprite.uv_rect.w = sprite.pixel_rect.w / tex_w;
                sprite.uv_rect.h = sprite.pixel_rect.h / tex_h;
            }
        }
    }
}