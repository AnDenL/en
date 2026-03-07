use eframe::egui;
use eframe::egui::FontId;
use std::env;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use en_ui::theme;

use crate::inspector::draw_json_inspector;

mod inspector;

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
            .with_decorations(false)
            .with_title(format!("En Editor - {}", project_path)),
        ..Default::default()
    };

    eframe::run_native(
        "En Editor",
        options,
        Box::new(|cc| {
            en_ui::theme::setup(&cc.egui_ctx);

            let wgpu_state = cc.wgpu_render_state.as_ref().expect("Eframe must run with WGPU!");
            let device = std::sync::Arc::new(wgpu_state.device.clone());
            let queue = std::sync::Arc::new(wgpu_state.queue.clone());
            let target_format = wgpu_state.target_format;

            let renderer = en_core::renderer::Renderer::new_for_editor(device, queue, target_format);

            Ok(Box::new(EditorApp::new(project_path, renderer)))
        }),
    )
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

struct EditorApp {
    project_path: String,
    scene_path: String,
    scene: en_core::scene::Scene,
    selected_entity: Option<usize>,
    
    renderer: en_core::renderer::Renderer,
    viewport_texture: Option<wgpu::Texture>,
    viewport_texture_id: Option<egui::TextureId>,

    current_asset_path: PathBuf,
    available_components: std::collections::HashMap<String, serde_json::Value>,
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
        
        let mut available_components = std::collections::HashMap::new();
        for template in en_core::inventory::iter::<en_core::ComponentTemplate> {
            available_components.insert(template.name.to_string(), (template.generator)());
        }

        Self {
            project_path: project_path.clone(),
            scene_path: scene_path.to_str().unwrap().to_string(),
            scene, 
            selected_entity: None,
            renderer, 
            viewport_texture: None,
            viewport_texture_id: None,
            current_asset_path: PathBuf::from(&project_path),
            available_components,
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.add_space(4.0);

            let header_height = 28.0; 
            let (header_rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), header_height), 
                egui::Sense::hover() 
            );

            let mut left_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(header_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center))
            );
            left_ui.add_space(8.0);
            left_ui.label(egui::RichText::new("💡 En Editor").font(FontId::proportional(16.0)).strong().color(en_ui::theme::ACCENT));
            left_ui.label(egui::RichText::new("|").color(en_ui::theme::TEXT_MUTED));
            left_ui.label(egui::RichText::new(&self.project_path).font(FontId::proportional(12.0)).color(en_ui::theme::TEXT_MUTED));

            let mut right_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(header_rect)
                    .layout(egui::Layout::right_to_left(egui::Align::Center))
            );
            right_ui.add_space(8.0);
            
            let close_btn = right_ui.add(egui::Button::new(egui::RichText::new("❌").size(14.0)).fill(egui::Color32::TRANSPARENT));
            let max_btn = right_ui.add(egui::Button::new(egui::RichText::new("🗖").size(14.0)).fill(egui::Color32::TRANSPARENT));
            let min_btn = right_ui.add(egui::Button::new(egui::RichText::new("—").size(14.0)).fill(egui::Color32::TRANSPARENT));

            if close_btn.clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
            if max_btn.clicked() {
                let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
            }
            if min_btn.clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true)); }

            let mut center_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(header_rect)
                    .layout(egui::Layout::top_down(egui::Align::Center))
            );
            center_ui.add_space((header_height - 22.0) / 2.0); 
            
            let play_btn = center_ui.button("▶ Play Game");
            if play_btn.clicked() {
                std::process::Command::new("cargo")
                    .arg("run")
                    .arg("-p")
                    .arg("en_runner") 
                    .arg("--")
                    .arg(&self.project_path) 
                    .spawn()
                    .expect("Unable to launch the game");
            }

            let any_btn_hovered = close_btn.hovered() || max_btn.hovered() || min_btn.hovered() || play_btn.hovered();
            
            if ui.rect_contains_pointer(header_rect) && !any_btn_hovered {
                if ctx.input(|i| i.pointer.primary_down()) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
            }

            ui.add_space(4.0);
        });

        egui::SidePanel::left("scene_tree_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("Scene").color(en_ui::theme::ACCENT));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("💾 Save").clicked() {
                            self.scene.save(&self.scene_path);
                            println!("Scene saved у: {:?}", self.scene_path);
                        }
                    });
                });
                ui.separator();
                if ui.button("➕ Add Entity").clicked() {
                    let mut components = std::collections::HashMap::new();
                    components.insert("Transform".to_string(), serde_json::json!({
                        "x": 0.0, "y": 0.0, "rotation": 0.0
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

                        let mut comp_to_remove = None;

                        for (comp_name, comp_value) in entity.components.iter_mut() {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(comp_name).strong().color(theme::ACCENT_BRIGHT));
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button(egui::RichText::new("🗑").color(egui::Color32::RED)).clicked() {
                                            comp_to_remove = Some(comp_name.clone());
                                        }
                                    });
                                });
                                ui.separator();
                                
                                draw_json_inspector(ui, comp_name, comp_value);
                            });
                            ui.add_space(5.0);
                        }

                        if let Some(name) = comp_to_remove {
                            entity.components.remove(&name);
                        }

                        ui.add_space(10.0);

                        ui.menu_button("➕ Add Component", |ui| {
                            for (name, template) in &self.available_components {
                                if !entity.components.contains_key(name) {
                                    if ui.button(name).clicked() {
                                        entity.components.insert(name.clone(), template.clone());
                                        ui.close();
                                    }
                                }
                            }
                        });
                    }
                } else {
                    ui.label(egui::RichText::new("No entity selected").color(theme::TEXT_MUTED));
                }
            });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(200.0)
            .min_height(100.0) 
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("Assets").color(en_ui::theme::ACCENT));
                    ui.add_space(10.0);

                    let is_root = self.current_asset_path == std::path::Path::new(&self.project_path);
                    
                    if !is_root {
                        if ui.button("⬆ Up").clicked() {
                            if let Some(parent) = self.current_asset_path.parent() {
                                self.current_asset_path = parent.to_path_buf();
                            }
                        }
                    }

                    let display_path = if is_root {
                        "res://".to_string()
                    } else {
                        if let Ok(rel) = self.current_asset_path.strip_prefix(&self.project_path) {
                            format!("res://{}", rel.display())
                        } else {
                            self.current_asset_path.display().to_string()
                        }
                    };
                    ui.label(egui::RichText::new(display_path).color(en_ui::theme::TEXT_MUTED));
                });
                
                ui.separator();
                
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if let Ok(entries) = std::fs::read_dir(&self.current_asset_path) {
                            let mut entries: Vec<_> = entries.flatten().collect();
                            entries.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));

                            for entry in entries {
                                let path = entry.path();
                                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                                
                                let (icon, color) = if path.is_dir() {
                                    ("📁", en_ui::theme::ACCENT)
                                } else {
                                    ("📄", en_ui::theme::TEXT_MAIN)
                                };

                                let file_btn = ui.add(
                                    egui::Button::new(
                                        egui::RichText::new(format!("{} {}", icon, file_name)).color(color)
                                    ).fill(en_ui::theme::CARD_BG)
                                );

                                if file_btn.double_clicked() && path.is_dir() {
                                    self.current_asset_path = path;
                                }
                            }
                        } else {
                            ui.label(egui::RichText::new("Directory reading error").color(egui::Color32::RED));
                        }
                    });
                });
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
                
                if let Some(old_id) = self.viewport_texture_id {
                    wgpu_state.renderer.write().free_texture(&old_id);
                }
                
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
    
                self.renderer.update_camera_buffer();

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

