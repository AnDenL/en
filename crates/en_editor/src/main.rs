use eframe::egui;
use std::env;

use crate::config::*;

mod app;
mod panels;
mod config;

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

    // 1. Setup Eframe options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_decorations(false) // We draw our own custom Top Bar
            .with_title(format!("En Editor - {}", project_path)),
        ..Default::default()
    };

    // 2. Run the application
    eframe::run_native(
        "En Editor",
        options,
        Box::new(|cc| {
            // Apply our custom dark theme to the egui context
            en_ui::theme::setup(&cc.egui_ctx);

            // Extract WGPU context from eframe.
            // This is crucial because our Engine needs the exact same device and queue
            // to render directly into the eframe UI window.
            let wgpu_state = cc.wgpu_render_state.as_ref().expect("Eframe must run with WGPU!");
            let device = std::sync::Arc::new(wgpu_state.device.clone());
            let queue = std::sync::Arc::new(wgpu_state.queue.clone());
            let target_format = wgpu_state.target_format;

            // Initialize our EditorApp which handles the Engine and the UI state
            Ok(Box::new(app::EditorApp::new(project_path, device, queue, target_format)))
        }),
    )
}