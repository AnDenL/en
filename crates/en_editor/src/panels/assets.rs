use eframe::egui;
use crate::app::EditorApp;

pub fn draw(ui: &mut egui::Ui, app: &mut EditorApp) {
    // Top bar of the Assets tab
    ui.horizontal(|ui| {
        let is_root = app.ui_state.current_asset_path == std::path::Path::new(&app.ui_state.project_path);
        
        if !is_root && ui.button("⬆ Up").clicked() {
            if let Some(parent) = app.ui_state.current_asset_path.parent() {
                app.ui_state.current_asset_path = parent.to_path_buf();
            }
        }
        
        let display_path = if is_root { "res://".into() } else {
            app.ui_state.current_asset_path.strip_prefix(&app.ui_state.project_path)
                .map(|p| format!("res://{}", p.display()))
                .unwrap_or_else(|_| app.ui_state.current_asset_path.display().to_string())
        };
        ui.label(egui::RichText::new(display_path).color(en_ui::theme::TEXT_MUTED));
    });

    ui.separator();

    // Refresh cache if directory changed
    if app.ui_state.current_asset_path != app.ui_state.last_asset_path {
        app.refresh_asset_cache();
    }

    // Scrollable content
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            let mut new_path_to_open = None;

            for (path, file_name, is_dir) in &app.ui_state.asset_cache {
                let (icon, color) = if *is_dir {
                    ("📁", en_ui::theme::ACCENT)
                } else {
                    ("📄", en_ui::theme::TEXT_MAIN)
                };

                let file_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new(format!("{} {}", icon, file_name)).color(color)
                    ).fill(en_ui::theme::CARD_BG)
                );

                if file_btn.double_clicked() && *is_dir {
                    new_path_to_open = Some(path.clone());
                }
            }

            // Apply path change after iteration
            if let Some(new_path) = new_path_to_open {
                app.ui_state.current_asset_path = new_path;
            }
        });
    });
}