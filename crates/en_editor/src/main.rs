use eframe::egui;
use eframe::egui::FontId;
use std::env;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use en_ui::theme;
use std::sync::{Arc, Mutex};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;

use crate::inspector::draw_typed_inspector;

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
    component_schemas: std::collections::HashMap<String, serde_json::Value>,
    active_tab: BottomTab,
    logs: Arc<Mutex<Vec<String>>>,
    game_process: Option<std::process::Child>,

    asset_cache: Vec<(PathBuf, String, bool)>, 
    last_asset_path: PathBuf,
    build_receiver: Option<Receiver<bool>>,
    is_building: Arc<AtomicBool>,
}

#[derive(PartialEq)]
enum BottomTab {
    Assets,
    Console,
}

impl EditorApp {
    fn new(project_path: String, renderer: en_core::renderer::Renderer) -> Self {
        let project_file = std::path::Path::new(&project_path).join("en_project.json");
        let project_json: serde_json::Value = if let Ok(data) = std::fs::read_to_string(&project_file) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            serde_json::json!({})
        };

        let entry_scene = if let Ok(data) = std::fs::read_to_string(&project_file) {
            let json: serde_json::Value = serde_json::from_str(&data).unwrap_or_default();
            json["entry_scene"].as_str().unwrap_or("main.scene").to_string()
        } else {
            "main.scene".to_string()
        };

        let loader_path = format!("{}/", project_path);
        let loader = en_core::assets::AssetLoader::new(&loader_path);

        let scene = pollster::block_on(en_core::scene::Scene::load(&loader, &entry_scene))
            .unwrap_or_else(|| {
                println!("[Editor error] Can't load scene {}", entry_scene);
                en_core::scene::Scene { entities: vec![] }
            });

        let scene_path = std::path::Path::new(&project_path).join(&entry_scene);
        
        let mut available_components = std::collections::HashMap::new();
        let mut component_schemas = std::collections::HashMap::new();
        for template in en_core::inventory::iter::<en_core::ComponentTemplate> {
            available_components.insert(template.name.to_string(), (template.generator)());
            component_schemas.insert(template.name.to_string(), (template.schema)());
        }

        let project_name = project_json["project_name"]
            .as_str()
            .unwrap_or("game")
            .to_lowercase()
            .replace("-", "_")
            .replace(" ", "_");

        let lib_path = format!("{}/target/debug/deps/lib{}.so", project_path, project_name);
        unsafe {
            if let Ok(lib) = libloading::Library::new(lib_path.clone()) {
                let func: Result<libloading::Symbol<unsafe extern "C" fn() -> *mut en_core::PluginRegistry>, _> = 
                    lib.get(b"en_get_plugin_registry");
                    
                if let Ok(get_registry) = func {
                    let registry_ptr = get_registry();
                    let registry = Box::from_raw(registry_ptr); 
                    for comp in registry.components {
                        available_components.insert(comp.name.to_string(), (comp.generator)());
                        println!("Component loaded: {}", comp.name);
                    }
                }
                std::mem::forget(lib); 
            } else {
                println!("⚠ Can't load DLL {}", lib_path);
            }
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
            component_schemas,
            logs: Arc::new(Mutex::new(Vec::new())),
            active_tab: BottomTab::Assets,
            game_process: None,
            asset_cache: Vec::new(), 
            last_asset_path: PathBuf::from("."),
            build_receiver: None,
            is_building: Arc::new(AtomicBool::new(false)),
        }
    }

    fn log(&self, message: &str) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(format!("[Editor] {}", message));
        }
    }

    fn rebuild_game_library(&mut self, ctx: &egui::Context) {
        if self.is_building.load(Ordering::SeqCst) {
            self.log("⚠ Build already started!");
            return;
        }

        self.is_building.store(true, Ordering::SeqCst);
        self.log("🔨 Building...");

        let (tx, rx) = std::sync::mpsc::channel();
        self.build_receiver = Some(rx);

        let project_path = self.project_path.clone();
        let is_building_clone = self.is_building.clone();
        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            let status = std::process::Command::new("cargo")
                .arg("build")
                .arg("--lib") 
                .current_dir(project_path)
                .status();

            let success = match status {
                Ok(s) => s.success(),
                _ => false,
            };

            let _ = tx.send(success);
            is_building_clone.store(false, Ordering::SeqCst);
            ctx_clone.request_repaint(); 
        });
    }

    fn reload_dll(&mut self) {
        let mut available_components = std::collections::HashMap::new();
        for template in en_core::inventory::iter::<en_core::ComponentTemplate> {
            available_components.insert(template.name.to_string(), (template.generator)());
        }

        let project_dir = std::path::Path::new(&self.project_path);
        let json_path = project_dir.join("en_project.json");
        
        let mut proj_name = String::from("game");
        if let Ok(data) = std::fs::read_to_string(&json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(n) = json["project_name"].as_str() {
                    proj_name = n.to_lowercase().replace("-", "_").replace(" ", "_");
                }
            }
        }

        let lib_name = format!("lib{}.so", proj_name);
        let lib_path = project_dir.join("target").join("debug").join(&lib_name);
        let lib_path_str = lib_path.to_str().unwrap().to_string();

        unsafe {
            if let Ok(lib) = libloading::Library::new(lib_path_str) {
                let func: Result<libloading::Symbol<unsafe extern "C" fn() -> *mut en_core::PluginRegistry>, _> = 
                    lib.get(b"en_get_plugin_registry");
                    
                if let Ok(get_registry) = func {
                    let registry_ptr = get_registry();
                    let registry = Box::from_raw(registry_ptr); 
                    for comp in registry.components {
                        available_components.insert(comp.name.to_string(), (comp.generator)());
                        self.log(&format!("Component loaded: {}", comp.name));
                    }
                }
                std::mem::forget(lib); 
            } else {
                self.log("⚠ Can't load DLL");
            }
        }
        self.available_components = available_components;
    }

    fn start_game_process(&mut self, ctx: &egui::Context) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.clear();
            logs.push("--- Starting the game ---".to_string());
        }

        if let Some(mut old_process) = self.game_process.take() {
            let _ = old_process.kill();
            let _ = old_process.wait();
        }

        let mut child = std::process::Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("game")
            .current_dir(&self.project_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Unable to launch the game");

        if let Some(stdout) = child.stdout.take() {
            let logs_clone = self.logs.clone();
            let ctx_clone = ctx.clone();
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let mut reader = BufReader::new(stdout);
                let mut buf = Vec::new();
                while let Ok(n) = reader.read_until(b'\n', &mut buf) {
                    if n == 0 { break; } 
                    let line = String::from_utf8_lossy(&buf).trim_end().to_string();
                    if let Ok(mut logs) = logs_clone.lock() {
                        logs.push(format!("ℹ {}", line));
                        if logs.len() > 300 { 
                            logs.remove(0);
                        }
                    }
                    }
                    ctx_clone.request_repaint();
                    buf.clear();
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let logs_clone = self.logs.clone();
            let ctx_clone = ctx.clone();
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let mut reader = BufReader::new(stderr);
                let mut buf = Vec::new();
                while let Ok(n) = reader.read_until(b'\n', &mut buf) {
                    if n == 0 { break; } 
                    let line = String::from_utf8_lossy(&buf).trim_end().to_string();
                    
                    if let Ok(mut logs) = logs_clone.lock() {
                        let is_cargo_status = line.trim_start().starts_with("Compiling") || 
                                            line.trim_start().starts_with("Finished") || 
                                            line.trim_start().starts_with("Running") || 
                                            line.trim_start().starts_with("Checking");

                        if is_cargo_status {
                            logs.push(format!("⚙ {}", line));
                        } else if line.trim_start().starts_with("error") {
                            logs.push(format!("❌ {}", line));
                        } else {
                            logs.push(format!("⚠ {}", line));
                        }

                        if logs.len() > 300 { 
                            logs.remove(0);
                        }
                    }
                    ctx_clone.request_repaint();
                    buf.clear();
                }
            });
        }
        self.game_process = Some(child);
    }

    fn refresh_asset_cache(&mut self) {
        self.asset_cache.clear();
        if let Ok(entries) = std::fs::read_dir(&self.current_asset_path) {
            let mut entries_vec: Vec<_> = entries.flatten().collect();
            entries_vec.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));

            for entry in entries_vec {
                let path = entry.path();
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let is_dir = path.is_dir();
                self.asset_cache.push((path, file_name, is_dir));
            }
        }
        self.last_asset_path = self.current_asset_path.clone();
    }

    fn prepare_scene_instances(&self) -> Vec<en_core::renderer::InstanceRaw> {
        let mut instances = Vec::new();

        for (index, entity) in self.scene.entities.iter().enumerate() {
            let sprite_data = match entity.components.get("Sprite") {
                Some(s) => s,
                None => continue,
            };

            let (mut x, mut y, mut rot) = (0.0, 0.0, 0.0);
            if let Some(t) = entity.components.get("Transform") {
                x = t["x"].as_f64().unwrap_or(0.0) as f32;
                y = t["y"].as_f64().unwrap_or(0.0) as f32;
                rot = t["rotation"].as_f64().unwrap_or(0.0) as f32;
            }

            let mut color = [1.0, 1.0, 1.0, 1.0];
            if let Some(c) = sprite_data["color"].as_array() {
                color = [
                    c.get(0).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                    c.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                    c.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                    c.get(3).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                ];
            }

            let cos_r = rot.cos();
            let sin_r = rot.sin();
            
            let model = [
                [cos_r,  sin_r, 0.0, 0.0],
                [-sin_r, cos_r, 0.0, 0.0],
                [0.0,    0.0,   1.0, 0.0],
                [x,      y,     0.0, 1.0],
            ];

            if self.selected_entity == Some(index) {
                let outline_scale = 1.15; 
                let outline_model = [
                    [cos_r * outline_scale,  sin_r * outline_scale, 0.0, 0.0],
                    [-sin_r * outline_scale, cos_r * outline_scale, 0.0, 0.0],
                    [0.0,    0.0,   1.0, 0.0],
                    [x,      y,     0.0, 1.0],
                ];
                
                let outline_color = [1.0, 0.8, 0.0, 1.0]; 

                instances.push(en_core::renderer::InstanceRaw { 
                    model: outline_model, 
                    color: outline_color 
                });
            }

            instances.push(en_core::renderer::InstanceRaw { model, color });
        }

        instances
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.build_receiver {
        if let Ok(success) = rx.try_recv() {
            self.build_receiver = None;
            if success {
                self.log("✅ Build successful! Loading DLL...");
                self.reload_dll();
            } else {
                self.log("❌ Build error!");
            }
        }
    }
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

            let mut play_btn_res = None;
            let mut build_btn_res = None;

            ui.scope_builder(egui::UiBuilder::new().max_rect(header_rect), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space((header_height - 22.0) / 2.0); 

                    ui.horizontal(|ui| {
                        let total_extra_space = ui.available_width() - 200.0;
                        ui.add_space(total_extra_space / 2.0);

                        ui.spacing_mut().item_spacing.x = 10.0;

                        let p_res = ui.button("▶ Play Game");
                        if p_res.clicked() {
                            self.start_game_process(ctx);
                        }
                        play_btn_res = Some(p_res);

                        let b_res = ui.button("🔨 Build");
                        if b_res.clicked() {
                            self.rebuild_game_library(ctx);
                        }
                        build_btn_res = Some(b_res);
                    });
                });
            });

            let play_hovered = play_btn_res.map_or(false, |r| r.hovered());
            let build_hovered = build_btn_res.map_or(false, |r| r.hovered());

            let any_btn_hovered = close_btn.hovered() 
                || max_btn.hovered() 
                || min_btn.hovered() 
                || play_hovered 
                || build_hovered;
                                            
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
                                
                                draw_typed_inspector(ui, comp_value, self.component_schemas.get(comp_name).expect("Draw inspector error"));
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
                    ui.selectable_value(&mut self.active_tab, BottomTab::Assets, "📁 Assets");
                    ui.selectable_value(&mut self.active_tab, BottomTab::Console, "💻 Console");
                    
                    ui.separator();

                    match self.active_tab {
                        BottomTab::Assets => {
                            let is_root = self.current_asset_path == std::path::Path::new(&self.project_path);
                            if !is_root && ui.button("⬆ Up").clicked() {
                                if let Some(parent) = self.current_asset_path.parent() {
                                    self.current_asset_path = parent.to_path_buf();
                                }
                            }
                            let display_path = if is_root { "res://".into() } else {
                                self.current_asset_path.strip_prefix(&self.project_path)
                                    .map(|p| format!("res://{}", p.display()))
                                    .unwrap_or_else(|_| self.current_asset_path.display().to_string())
                            };
                            ui.label(egui::RichText::new(display_path).color(theme::TEXT_MUTED));
                        }
                        BottomTab::Console => {
                            if ui.button("🗑 Clear").clicked() {
                                if let Ok(mut logs) = self.logs.lock() { logs.clear(); }
                            }
                        }
                    }
                });
                
                ui.separator();
                
                match self.active_tab {
                    BottomTab::Assets => {
                        if self.current_asset_path != self.last_asset_path {
                            self.refresh_asset_cache();
                        }
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
                    }
                    BottomTab::Console => {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .stick_to_bottom(true) 
                            .show(ui, |ui| {
                                if let Ok(logs) = self.logs.lock() {
                                    for log in logs.iter() {
                                        if log.starts_with("⚠") {
                                            ui.label(egui::RichText::new(log).color(egui::Color32::LIGHT_RED));
                                        } else {
                                            ui.label(egui::RichText::new(log).color(theme::TEXT_MAIN));
                                        }
                                    }
                                }
                            });
                    }
                }
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

                let instances = self.prepare_scene_instances();
                
                self.renderer.render_to_view(&instances, &view);
                
                let image = egui::Image::new(egui::load::SizedTexture::new(id, size))
                    .sense(egui::Sense::drag());
                let response = ui.add(image);
                let mut camera_changed = false;
                self.renderer.render_to_view(&instances, &view);

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
impl Drop for EditorApp {
    fn drop(&mut self) {
        if let Some(mut process) = self.game_process.take() {
            let _ = process.kill();
        }
    }
}