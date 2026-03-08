use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use chrono::Local;
use en_ui::theme;

const ENGINE_PATH: &str = "/home/andenl/Documents/GitHub/en/crates"; 
const CURRENT_ENGINE_VERSION: &str = "1.0";

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_title("En Hub"),
        ..Default::default()
    };
    
    eframe::run_native(
        "En Hub",
        options,
        Box::new(|cc| {
            en_ui::theme::setup(&cc.egui_ctx);
            Ok(Box::new(HubApp::new()))
        }),
    )
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct ProjectRecord {
    name: String,
    path: String,
    created_at: String,
    is_favorite: bool,
}

struct HubApp {
    projects: Vec<ProjectRecord>,
    show_new_project: bool,
    new_project_name: String,
    new_project_path: String,
    status_message: String,
}

impl HubApp {
    fn new() -> Self {
        Self {
            projects: load_projects(),
            show_new_project: false,
            new_project_name: String::new(),
            new_project_path: String::new(),
            status_message: String::new(),
        }
    }

    fn add_and_save_project(&mut self, record: ProjectRecord) {
        if !self.projects.iter().any(|p| p.path == record.path) {
            self.projects.push(record);
            save_projects(&self.projects);
        } else {
            self.status_message = "Project already exists in the Hub!".to_string();
        }
    }
}

impl eframe::App for HubApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(12.0))
            .show(ctx, |ui| {
                ui.available_rect_before_wrap();
                
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("En projects")
                            .color(theme::ACCENT)
                            .size(32.0) 
                            .strong()
                    );

                    ui.add_space(20.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let close_btn = ui.button(egui::RichText::new(" ❌ ").color(theme::ACCENT_BRIGHT).size(24.0));
                        if close_btn.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        
                        ui.add_space(15.0);

                        let import_btn = ui.button(egui::RichText::new(" 📥 Import ").color(theme::ACCENT).size(20.0));
                        if import_btn.clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                let json_path = folder.join("en_project.json");
                                if json_path.exists() {
                                    let mut proj_name = folder.file_name().unwrap_or_default().to_string_lossy().into_owned();
                                    if let Ok(data) = fs::read_to_string(&json_path) {
                                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                                            if let Some(n) = json["project_name"].as_str() {
                                                proj_name = n.to_string();
                                            }
                                        }
                                    }
                                    
                                    let current_date = Local::now().format("%d.%m.%Y").to_string();
                                    self.add_and_save_project(ProjectRecord {
                                        name: proj_name,
                                        path: folder.to_string_lossy().into_owned(),
                                        created_at: current_date,
                                        is_favorite: false,
                                    });
                                } else {
                                    self.status_message = "Error: Selected folder is not an En Engine project!".to_string();
                                }
                            }
                        }

                        ui.add_space(10.0);

                        let new_btn = ui.button(egui::RichText::new(" ➕ New ").color(theme::ACCENT).size(20.0));
                        if new_btn.clicked() {
                            self.show_new_project = !self.show_new_project;
                            self.status_message.clear();
                        }

                        if !self.status_message.is_empty() && !self.show_new_project {
                            ui.label(egui::RichText::new(&self.status_message).color(theme::ERROR).size(16.0));
                        }

                        let drag_response = ui.allocate_response(ui.available_size(), egui::Sense::click());
                        if drag_response.is_pointer_button_down_on() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(16.0))
            .show(ctx, |ui| {
                let mut remove_idx = None;
                let mut relocate_idx = None;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, proj) in self.projects.iter_mut().enumerate() {
                        let item_width = ui.available_width();
                        let rect_min = ui.next_widget_position();
                        let predicted_rect = egui::Rect::from_min_size(rect_min, egui::vec2(item_width, 64.0));
                        
                        let is_hovered = ui.rect_contains_pointer(predicted_rect);
                        let fill_color = if is_hovered { theme::CARD_HOVER } else { theme::CARD_BG };

                        let frame = egui::Frame::new()
                            .fill(fill_color)
                            .corner_radius(6.0)
                            .inner_margin(16.0); 
                        
                        let proj_path = PathBuf::from(&proj.path);
                        let json_path = proj_path.join("en_project.json");
                        let is_valid = json_path.exists();

                        let response = frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🎮").size(36.0)); 
                                ui.add_space(12.0);
                                
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&proj.name).color(theme::ACCENT).size(24.0).strong()); 
                                    ui.label(egui::RichText::new(&proj.path).color(theme::TEXT_MUTED).size(14.0));
                                });
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(egui::RichText::new("🗑").size(20.0).color(theme::ERROR))
                                        .on_hover_text("Remove from list")
                                        .clicked() 
                                    {
                                        remove_idx = Some(idx);
                                    }

                                    ui.add_space(12.0);

                                    if is_valid {
                                        let fav_text = if proj.is_favorite { "♥" } else { "♡" };
                                        let fav_btn = ui.add(egui::Button::new(
                                            egui::RichText::new(fav_text).color(theme::ACCENT).size(24.0)
                                        ).fill(egui::Color32::TRANSPARENT));
                                        
                                        if fav_btn.clicked() {
                                            proj.is_favorite = !proj.is_favorite;
                                        }
                                        let mut proj_version = String::from("Unknown");
                                        if let Ok(data) = fs::read_to_string(&json_path) {
                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                                                if let Some(v) = json["engine_version"].as_str() {
                                                    proj_version = v.to_string();
                                                }
                                            }
                                        }

                                        if proj_version != CURRENT_ENGINE_VERSION {
                                            if ui.button(egui::RichText::new("⬆ Upgrade").size(16.0).color(theme::WARNING)).on_hover_text(format!("Upgrade from v{} to v{}", proj_version, CURRENT_ENGINE_VERSION)).clicked() {
                                                if let Ok(data) = fs::read_to_string(&json_path) {
                                                    if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&data) {
                                                        json["engine_version"] = serde_json::json!(CURRENT_ENGINE_VERSION);
                                                        let _ = fs::write(&json_path, serde_json::to_string_pretty(&json).unwrap());
                                                    }
                                                }
                                            }
                                            ui.label(egui::RichText::new(format!("v{}", proj_version)).color(theme::WARNING).size(16.0));
                                        }
                                    } else {
                                        if ui.button(egui::RichText::new("📁 Relocate").size(16.0)).clicked() {
                                            relocate_idx = Some(idx);
                                        }
                                        ui.label(egui::RichText::new("Missing!").color(theme::ERROR).size(16.0));
                                    }
                                });
                            });
                        }).response;

                        if is_valid {
                            let interact = ui.interact(response.rect, ui.id().with(&proj.path), egui::Sense::click());
                            if interact.double_clicked() {
                                println!("LAUNCH PROJECT: {}", proj.path);
                                std::process::Command::new("cargo")
                                    .args(["run", "-p", "en_editor", "--", &proj.path])
                                    .spawn()
                                    .expect("Failed to start en_editor");
                            }
                        }
                        
                        ui.add_space(8.0);
                    }
                });

                if let Some(idx) = remove_idx {
                    self.projects.remove(idx);
                    save_projects(&self.projects);
                }

                if let Some(idx) = relocate_idx {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        self.projects[idx].path = folder.to_string_lossy().into_owned();
                        save_projects(&self.projects);
                    }
                }
            });

        if self.show_new_project {
            egui::Window::new("new_project_window")
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .default_pos(ctx.content_rect().center())
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_size([400.0, 0.0]) 
                .frame(egui::Frame::window(&ctx.style())
                    .fill(theme::CARD_BG)
                    .inner_margin(16.0)
                    .corner_radius(8.0)
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Create New Project").size(22.0).strong().color(theme::ACCENT));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new(" ❌ ").size(16.0).color(theme::ACCENT_BRIGHT)).clicked() {
                                self.show_new_project = false;
                                self.status_message.clear();
                            }
                        });
                    });
                    
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(12.0);
                    
                    let label_width = 50.0;

                    ui.horizontal(|ui| {
                        ui.add_sized([label_width, 20.0], egui::Label::new(egui::RichText::new("Name:").size(16.0).color(theme::TEXT_MUTED)));
                        ui.add(egui::TextEdit::singleline(&mut self.new_project_name)
                            .margin(egui::vec2(8.0, 8.0))
                            .desired_width(f32::INFINITY));
                    });

                    if !self.new_project_name.is_empty() {
                        let safe = sanitize_package_name(&self.new_project_name);
                        if safe != self.new_project_name.to_lowercase() {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.add_space(label_width + 12.0); 
                                ui.label(egui::RichText::new(format!("⚠ Cargo name: {}", safe))
                                    .size(12.0)
                                    .color(theme::WARNING));
                            });
                        }
                    }
                    
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.add_sized([label_width, 20.0], egui::Label::new(egui::RichText::new("Path:").size(16.0).color(theme::TEXT_MUTED)));
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(egui::RichText::new(" 📁 ").size(18.0)).min_size(egui::vec2(40.0, 32.0))).clicked() {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    self.new_project_path = folder.to_string_lossy().into_owned();
                                }
                            }

                            ui.add(egui::TextEdit::singleline(&mut self.new_project_path)
                                .margin(egui::vec2(8.0, 8.0))
                                .desired_width(f32::INFINITY));
                        });
                    });

                    ui.add_space(20.0);
                    
                    let create_btn = ui.add_sized(
                        [ui.available_width(), 40.0],
                        egui::Button::new(egui::RichText::new("Create Project").size(18.0).strong())
                            .fill(theme::ACCENT)
                    );

                    if create_btn.clicked() {
                        if self.new_project_name.trim().is_empty() || self.new_project_path.trim().is_empty() {
                            self.status_message = "Error: Enter name and select location!".to_string();
                        } else {
                            let base_folder = PathBuf::from(&self.new_project_path);
                            let project_path = base_folder.join(&self.new_project_name);
                            
                            match create_project_structure(&project_path, &self.new_project_name) {
                                Ok(record) => {
                                    self.add_and_save_project(record);
                                    self.show_new_project = false;
                                    self.new_project_name.clear();
                                    self.new_project_path.clear();
                                    self.status_message.clear();
                                }
                                Err(e) => {
                                    self.status_message = format!("Error: {}", e);
                                }
                            }
                        }
                    }

                    if !self.status_message.is_empty() {
                        ui.add_space(10.0);
                        let msg_color = if self.status_message.starts_with("Error") { theme::ERROR } else { theme::WARNING };
                        ui.label(egui::RichText::new(&self.status_message).color(msg_color).size(14.0));
                    }
                });
        }
    }
}

fn create_project_structure(base_path: &Path, name: &str) -> Result<ProjectRecord, String> {
    if base_path.exists() {
        return Err("Folder with this name already exists!".into());
    }

    let safe_name = sanitize_package_name(name);
    if safe_name.replace('_', "").is_empty() {
        return Err("Invalid project name for Cargo!".into());
    }

    fs::create_dir_all(base_path.join("assets")).map_err(|e| e.to_string())?;
    fs::create_dir_all(base_path.join("src").join("scripts")).map_err(|e| e.to_string())?;

    let cargo_template = include_str!("../templates/Cargo.toml.template");
    let lib_template = include_str!("../templates/lib.rs.template");
    let main_template = include_str!("../templates/main.rs.template");
    let json_template = include_str!("../templates/en_project.json.template");

    let cargo_toml = cargo_template
        .replace("{project_name}", &safe_name)
        .replace("{engine_path}", ENGINE_PATH);

    let lib_rs = lib_template
        .replace("{project_name}", &safe_name);

    let en_project_json = json_template
        .replace("{project_name}", name)
        .replace("{engine_version}", CURRENT_ENGINE_VERSION);

    fs::write(base_path.join("Cargo.toml"), cargo_toml).map_err(|e| e.to_string())?;
    fs::write(base_path.join("src").join("lib.rs"), lib_rs).map_err(|e| e.to_string())?;
    fs::write(base_path.join("src").join("main.rs"), main_template).map_err(|e| e.to_string())?;
    fs::write(base_path.join("en_project.json"), en_project_json).map_err(|e| e.to_string())?;

    let base_scene = r#"{ "entities": [] }"#;
    fs::write(base_path.join("assets").join("main.scene"), base_scene).map_err(|e| e.to_string())?;

    let current_date = Local::now().format("%d.%m.%Y").to_string();

    Ok(ProjectRecord {
        name: name.to_string(),
        path: base_path.to_string_lossy().into_owned(),
        created_at: current_date,
        is_favorite: false,
    })
}

fn sanitize_package_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

fn get_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "en", "EnEngine") {
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).unwrap();
        config_dir.join("projects.json")
    } else {
        panic!("Could not find system folder for configs!");
    }
}

fn load_projects() -> Vec<ProjectRecord> {
    let path = get_config_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(projects) = serde_json::from_str(&data) {
                return projects;
            }
        }
    }
    Vec::new()
}

fn save_projects(projects: &[ProjectRecord]) {
    let path = get_config_path();
    if let Ok(data) = serde_json::to_string_pretty(projects) {
        let _ = fs::write(path, data);
    }
}