use eframe::egui;
use crate::app::EditorApp;

pub fn draw(ui: &mut egui::Ui, app: &mut EditorApp) {
    ui.horizontal(|ui| {
        if ui.button("🗑 Clear").clicked() {
            if let Ok(mut logs) = app.ui_state.logs.lock() { 
                logs.clear(); 
            }
        }
    });

    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true) 
        .show(ui, |ui| {
            if let Ok(logs) = app.ui_state.logs.lock() {
                for log in logs.iter() {
                    if log.starts_with("⚠") {
                        ui.label(egui::RichText::new(log).color(egui::Color32::LIGHT_RED));
                    } else if log.starts_with("❌") {
                        ui.label(egui::RichText::new(log).color(en_ui::theme::ERROR));
                    } else {
                        ui.label(egui::RichText::new(log).color(en_ui::theme::TEXT_MAIN));
                    }
                }
            }
        });
}