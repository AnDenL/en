use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use chrono::Local;
use en_ui::theme;

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
        self.projects.push(record);
        save_projects(&self.projects);
    }
}

impl eframe::App for HubApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.available_rect_before_wrap();
                
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("En projects")
                            .color(theme::ACCENT)
                            .size(28.0)
                            .strong()
                    );

                    ui.add_space(20.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let close_btn = ui.button(egui::RichText::new(" X ").color(theme::ACCENT_BRIGHT).size(24.0));
                        if close_btn.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        
                        ui.add_space(10.0);

                        let new_btn = ui.button(egui::RichText::new(" + New ").color(theme::ACCENT).size(20.0));
                        if new_btn.clicked() {
                            self.show_new_project = !self.show_new_project;
                            self.status_message.clear();
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for proj in &self.projects {
                        let item_width = ui.available_width();
                        let rect_min = ui.next_widget_position();
                        let predicted_rect = egui::Rect::from_min_size(rect_min, egui::vec2(item_width, 56.0));
                        
                        let is_hovered = ui.rect_contains_pointer(predicted_rect);
                        let fill_color = if is_hovered { theme::CARD_HOVER } else { theme::CARD_BG };

                        let frame = egui::Frame::new()
                            .fill(fill_color)
                            .corner_radius(4.0)
                            .inner_margin(12.0);
                        
                        let response = frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🎮").size(32.0));
                                ui.add_space(8.0);
                                
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&proj.name).color(theme::ACCENT).size(20.0).strong());
                                    ui.label(egui::RichText::new(&proj.path).color(theme::TEXT_MUTED).size(12.0));
                                });
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new(if proj.is_favorite { "♥" } else { "♡" })
                                        .color(theme::ACCENT).size(20.0));
                                    ui.label(egui::RichText::new(&proj.created_at).color(theme::TEXT_MUTED));
                                });
                            });
                        }).response;

                        let interact = ui.interact(response.rect, ui.id().with(&proj.path), egui::Sense::click());
                        if interact.double_clicked() {
                            println!("LAUNCH PROJECT: {}", proj.path);

                            std::process::Command::new("cargo")
                                .args(["run", "-p", "en_editor", "--", &proj.path])
                                .spawn()
                                .expect("Failed to start en_editor");
                        }
                        
                        ui.add_space(8.0);
                    }
                });
            });
        if self.show_new_project {
            egui::Window::new("Create New Project")
                .collapsible(false) 
                .resizable(false)  
                .default_pos(ctx.content_rect().center()) 
                .pivot(egui::Align2::CENTER_CENTER)
                .show(ctx, |ui| {
                    ui.set_width(350.0);
                    
                    ui.vertical(|ui| {
                        ui.add_space(8.0);
                        
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Name:").color(theme::TEXT_MUTED));
                            ui.text_edit_singleline(&mut self.new_project_name);
                        });
                        
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Location:").color(theme::TEXT_MUTED));
                            ui.text_edit_singleline(&mut self.new_project_path);
                            if ui.button(" 📁 ").clicked() {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    self.new_project_path = folder.to_string_lossy().into_owned();
                                }
                            }
                        });

                        ui.add_space(12.0);
                        
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                self.show_new_project = false;
                                self.status_message.clear();
                            }

                            let create_btn = ui.add(egui::Button::new(
                                egui::RichText::new(" Create Project ").strong()
                            ).fill(theme::ACCENT));

                            if create_btn.clicked() {
                                if self.new_project_name.trim().is_empty() || self.new_project_path.trim().is_empty() {
                                    self.status_message = "Enter name and select location!".to_string();
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
                        });

                        if !self.status_message.is_empty() {
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new(&self.status_message).color(egui::Color32::YELLOW));
                        }
                    });
                });
        }
    }
}

fn create_project_structure(base_path: &Path, name: &str) -> Result<ProjectRecord, String> {
    if base_path.exists() {
        return Err("Folder with this name already exists!".into());
    }

    fs::create_dir_all(base_path.join("assets")).map_err(|e| e.to_string())?;
    fs::create_dir_all(base_path.join("scripts")).map_err(|e| e.to_string())?;

    let project_data = serde_json::json!({
        "project_name": name,
        "engine_version": "0.1.0",
        "entry_scene": "main.scene"
    });

    let json_string = serde_json::to_string_pretty(&project_data).unwrap();
    fs::write(base_path.join("en_project.json"), json_string).map_err(|e| e.to_string())?;

    let current_date = Local::now().format("%d.%m.%Y").to_string();

    Ok(ProjectRecord {
        name: name.to_string(),
        path: base_path.to_string_lossy().into_owned(),
        created_at: current_date,
        is_favorite: false,
    })
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