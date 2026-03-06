use eframe::egui;
use std::env;

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

fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().collect();
    let project_path = if args.len() > 1 {
        args[1].clone()
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
        Box::new(|_| Box::new(EditorApp::new(project_path))),
    )
}

struct EditorApp {
    project_path: String,
}

impl EditorApp {
    fn new(project_path: String) -> Self {
        Self { project_path }
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
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save Scene").clicked() {}
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Project", |ui| {
                    if ui.button("Settings").clicked() {}
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(&self.project_path).color(theme::TEXT_MUTED));
                });
            });
        });

        egui::SidePanel::left("scene_tree_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("Scene").color(theme::ACCENT));
                ui.separator();
                ui.label(egui::RichText::new("Root Node").color(theme::TEXT_MAIN));
            });

        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("Inspector").color(theme::ACCENT));
                ui.separator();
                ui.label(egui::RichText::new("No entity selected").color(theme::TEXT_MUTED));
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
            let viewport_rect = ui.available_rect_before_wrap();
            ui.painter().rect_filled(viewport_rect, 0.0, theme::CARD_BG);
            
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("VIEWPORT").color(theme::CARD_HOVER).size(40.0).strong());
            });
        });
    }
}