use eframe::egui;
use app::{HubApp};

mod app;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
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