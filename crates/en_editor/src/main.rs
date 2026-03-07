use eframe::egui;
use std::env;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

pub mod theme {
    use eframe::egui::Color32;
    pub const BG: Color32 = Color32::from_rgb(30, 27, 46); 
    pub const CARD_BG: Color32 = Color32::from_rgb(42, 38, 64);
    pub const CARD_HOVER: Color32 = Color32::from_rgb(56, 51, 85);
    pub const ACCENT: Color32 = Color32::from_rgb(242, 166, 90);
    pub const ACCENT_BRIGHT: Color32 = Color32::from_rgb(255, 192, 133);
    pub const TEXT_MAIN: Color32 = Color32::from_rgb(234, 230, 240);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(154, 147, 166);
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct EditorConfig {
    last_project_path: Option<String>,
}

fn get_editor_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "en", "EnEngine") {
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).unwrap();
        config_dir.join("editor_config.json")
    } else {
        panic!("Could not find system folder for configs!");
    }
}

fn load_editor_config() -> EditorConfig {
    let path = get_editor_config_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str(&data) {
                return config;
            }
        }
    }
    EditorConfig::default()
}

fn save_editor_config(config: &EditorConfig) {
    let path = get_editor_config_path();
    if let Ok(data) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, data);
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut config = load_editor_config();

    let project_path = if args.len() > 1 {
        let path = args[1].clone();
        config.last_project_path = Some(path.clone());
        save_editor_config(&config);
        path
    } else if let Some(last_path) = config.last_project_path {
        last_path
    } else {
        "No project loaded".to_string()
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_title(format!("En Editor - {}", project_path)),
        ..Default::default()
    };

    eframe::run_native(
    "En Editor",
    options,
    Box::new(|cc| {
        let wgpu_state = cc.wgpu_render_state.as_ref().expect("Eframe must run with WGPU!");
        let device = std::sync::Arc::new(wgpu_state.device.clone());
        let queue = std::sync::Arc::new(wgpu_state.queue.clone());
        let target_format = wgpu_state.target_format;

        let renderer = en_core::renderer::Renderer::new_for_editor(device, queue, target_format);

        Ok(Box::new(EditorApp::new(project_path, renderer)))
    }),
)
}

struct EditorApp {
    project_path: String,
    scene_path: String,
    scene: en_core::scene::Scene,
    selected_entity: Option<usize>,
    
    renderer: en_core::renderer::Renderer,
    viewport_texture: Option<wgpu::Texture>,
    viewport_texture_id: Option<egui::TextureId>,
}
impl EditorApp {
    fn new(project_path: String, renderer: en_core::renderer::Renderer) -> Self {
        let project_file = std::path::Path::new(&project_path).join("project.json");
        let entry_scene = if let Ok(data) = std::fs::read_to_string(&project_file) {
            let json: serde_json::Value = serde_json::from_str(&data).unwrap_or_default();
            json["entry_scene"].as_str().unwrap_or("main.scene").to_string()
        } else {
            "main.scene".to_string()
        };

        let scene_path = std::path::Path::new(&project_path).join(&entry_scene);

        let scene = en_core::scene::Scene::load(scene_path.to_str().unwrap())
            .unwrap_or_else(|| en_core::scene::Scene { entities: vec![] });

        Self {
            project_path,
            scene_path: scene_path.to_str().unwrap().to_string(),
            scene,
            selected_entity: None,
            renderer, 
            viewport_texture: None,
            viewport_texture_id: None,
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = theme::BG;
        visuals.widgets.noninteractive.bg_fill = theme::BG;
        visuals.widgets.noninteractive.fg_stroke.color = theme::TEXT_MAIN;
        ctx.set_visuals(visuals);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui,|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("💾 Save Scene").clicked() {
                        self.scene.save(&self.scene_path);
                        println!("Scene saved у: {:?}", self.scene_path);
                    }
                    if ui.button("🚪 Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Run", |ui| {
                    if ui.button("▶ Play Game").clicked() {
                        std::process::Command::new("cargo")
                            .arg("run")
                            .arg("-p")
                            .arg("en_runner") 
                            .arg("--")
                            .arg(&self.project_path) 
                            .spawn()
                            .expect("Unable to launch the game");
                    }
                });
            });
        });

        egui::SidePanel::left("scene_tree_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("Scene").color(theme::ACCENT));
                ui.separator();

                if ui.button("➕ Add Entity").clicked() {
                    let mut components = std::collections::HashMap::new();
                    components.insert("Transform".to_string(), serde_json::json!({
                        "x": 0.0, "y": 0.0, "rotation": 0.0
                    }));
                    components.insert("Sprite".to_string(), serde_json::json!({
                        "color": [1.0, 1.0, 1.0, 1.0]
                    }));

                    let new_entity = en_core::scene::EntityData {
                        name: format!("Entity {}", self.scene.entities.len()),
                        components,
                    };
                    self.scene.entities.push(new_entity);
                    self.selected_entity = Some(self.scene.entities.len() - 1);
                }

                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, entity) in self.scene.entities.iter().enumerate() {
                        let is_selected = self.selected_entity == Some(index);
                        if ui.selectable_label(is_selected, &entity.name).clicked() {
                            self.selected_entity = Some(index);
                        }
                    }
                });
            });

        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("Inspector").color(theme::ACCENT));
                ui.separator();

                if let Some(index) = self.selected_entity {
                    if let Some(entity) = self.scene.entities.get_mut(index) {
                        
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Name:").color(theme::TEXT_MAIN));
                            ui.text_edit_singleline(&mut entity.name);
                        });

                        ui.add_space(10.0);

                        for (comp_name, comp_value) in entity.components.iter_mut() {
                            ui.group(|ui| {
                                draw_json_inspector(ui, comp_name, comp_value);
                            });
                            ui.add_space(5.0);
                        }
                    }
                } else {
                    ui.label(egui::RichText::new("No entity selected").color(theme::TEXT_MUTED));
                }
            });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(200.0)
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("Assets").color(theme::ACCENT));
                ui.separator();
                ui.label(egui::RichText::new("res://").color(theme::TEXT_MUTED));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let size = ui.available_size();
            let width = size.x.max(1.0) as u32;
            let height = size.y.max(1.0) as u32;

            let needs_recreate = self.viewport_texture.as_ref().map_or(true, |tex| {
                tex.size().width != width || tex.size().height != height
            });

            if needs_recreate {
                let wgpu_state = _frame.wgpu_render_state().expect("WGPU is not enabled in eframe!");
                
                let texture = self.renderer.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Viewport Texture"),
                    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8Unorm, 
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });

                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                
                let id = wgpu_state.renderer.write().register_native_texture(
                    &*self.renderer.device,
                    &view,
                    wgpu::FilterMode::Linear,
                );

                self.renderer.camera.update_aspect_ratio(width as f32, height as f32);

                self.viewport_texture = Some(texture);
                self.viewport_texture_id = Some(id);
            }

            if let (Some(texture), Some(id)) = (&self.viewport_texture, self.viewport_texture_id) {
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

                let mut instances = Vec::new();
                for entity in &self.scene.entities {
                    let mut x = 0.0;
                    let mut y = 0.0;
                    let mut rot = 0.0;
                    let mut color = [1.0, 1.0, 1.0, 1.0];

                    if let Some(t) = entity.components.get("Transform") {
                        x = t["x"].as_f64().unwrap_or(0.0) as f32;
                        y = t["y"].as_f64().unwrap_or(0.0) as f32;
                        rot = t["rotation"].as_f64().unwrap_or(0.0) as f32;
                    }

                    if let Some(s) = entity.components.get("Sprite") {
                        if let Some(c) = s["color"].as_array() {
                            color = [
                                c[0].as_f64().unwrap_or(1.0) as f32,
                                c[1].as_f64().unwrap_or(1.0) as f32,
                                c[2].as_f64().unwrap_or(1.0) as f32,
                                c[3].as_f64().unwrap_or(1.0) as f32,
                            ];
                        }
                    }

                    let cos_r = rot.cos();
                    let sin_r = rot.sin();
                    let model = [
                        [cos_r,  sin_r, 0.0, 0.0],
                        [-sin_r, cos_r, 0.0, 0.0],
                        [0.0,    0.0,   1.0, 0.0],
                        [x,      y,     0.0, 1.0],
                    ];

                    instances.push(en_core::renderer::InstanceRaw { model, color });
                }

                self.renderer.render_to_view(&instances, &view);
                
                let image = egui::Image::new(egui::load::SizedTexture::new(id, size))
                    .sense(egui::Sense::drag());
                let response = ui.add(image);
                let mut camera_changed = false;

                if response.dragged_by(egui::PointerButton::Secondary) {
                    let delta = response.drag_delta();
                    
                    let world_unit_per_pixel = (360.0 * 2.0 * self.renderer.camera.scale) / size.y;

                    self.renderer.camera.x -= delta.x * world_unit_per_pixel;
                    self.renderer.camera.y += delta.y * world_unit_per_pixel;
                    camera_changed = true;
                }

                if response.hovered() {
                    let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
                    if scroll != 0.0 {
                        let zoom_speed = 0.001;
                        self.renderer.camera.scale *= 1.0 - (scroll * zoom_speed);
                        
                        self.renderer.camera.scale = self.renderer.camera.scale.clamp(0.01, 100.0);
                        camera_changed = true;
                    }
                }

                if camera_changed {
                    self.renderer.camera.update_aspect_ratio(size.x, size.y);
                    self.renderer.update_camera_buffer();
                }
            }
        });
    }
}

fn draw_json_inspector(ui: &mut egui::Ui, name: &str, value: &mut serde_json::Value) -> bool {
    let mut changed = false;

    match value {
        serde_json::Value::Object(map) => {
            ui.collapsing(egui::RichText::new(name).strong(), |ui| {
                for (k, v) in map.iter_mut() {
                    if draw_json_inspector(ui, k, v) {
                        changed = true;
                    }
                }
            });
        }
        serde_json::Value::Array(arr) => {
            if arr.len() == 4 && arr.iter().all(|v| v.is_number()) {
                ui.horizontal(|ui| {
                    ui.label(name);
                    let r = arr[0].as_f64().unwrap_or(1.0) as f32;
                    let g = arr[1].as_f64().unwrap_or(1.0) as f32;
                    let b = arr[2].as_f64().unwrap_or(1.0) as f32;
                    let a = arr[3].as_f64().unwrap_or(1.0) as f32;
                    
                    let mut color = egui::Color32::from_rgba_unmultiplied(
                        (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (a * 255.0) as u8
                    );
                    
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        arr[0] = serde_json::json!(color.r() as f32 / 255.0);
                        arr[1] = serde_json::json!(color.g() as f32 / 255.0);
                        arr[2] = serde_json::json!(color.b() as f32 / 255.0);
                        arr[3] = serde_json::json!(color.a() as f32 / 255.0);
                        changed = true;
                    }
                });
            } else {
                ui.collapsing(name, |ui| {
                    for (i, v) in arr.iter_mut().enumerate() {
                        if draw_json_inspector(ui, &format!("[{}]", i), v) { changed = true; }
                    }
                });
            }
        }
        serde_json::Value::Number(num) => {
            ui.horizontal(|ui| {
                ui.label(name);
                if let Some(val) = num.as_f64() {
                    let mut f = val;
                    if ui.add(egui::DragValue::new(&mut f).speed(0.1)).changed() {
                        if num.is_i64() || num.is_u64() {
                            *num = serde_json::Number::from(f as i64);
                        } else {
                            *num = serde_json::Number::from_f64(f).unwrap();
                        }
                        changed = true;
                    }
                }
            });
        }
        serde_json::Value::String(s) => {
            ui.horizontal(|ui| {
                ui.label(name);
                if ui.text_edit_singleline(s).changed() { changed = true; }
            });
        }
        serde_json::Value::Bool(b) => {
            ui.horizontal(|ui| {
                ui.label(name);
                if ui.checkbox(b, "").changed() { changed = true; }
            });
        }
        serde_json::Value::Null => {
            ui.horizontal(|ui| { ui.label(name); ui.label("null"); });
        }
    }
    changed
}