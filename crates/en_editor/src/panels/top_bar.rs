use crate::app::EditorApp;
use eframe::egui;
use eframe::egui::FontId;

pub fn draw(ui: &mut egui::Ui, app: &mut EditorApp) {
    ui.add_space(4.0);

    let header_height = 28.0;
    let (header_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), header_height),
        egui::Sense::hover(),
    );

    // --- LEFT SECTION: Logo and Project Path ---
    let mut left_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(header_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    left_ui.add_space(8.0);
    left_ui.label(
        egui::RichText::new("💡 En Editor")
            .font(FontId::proportional(16.0))
            .strong()
            .color(en_ui::theme::ACCENT),
    );
    left_ui.label(egui::RichText::new("|").color(en_ui::theme::TEXT_MUTED));
    left_ui.label(
        egui::RichText::new(&app.ui_state.project_path)
            .font(FontId::proportional(12.0))
            .color(en_ui::theme::TEXT_MUTED),
    );

    // --- RIGHT SECTION: Window Controls ---
    let mut right_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(header_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    right_ui.add_space(8.0);

    let close_btn = right_ui.add(
        egui::Button::new(egui::RichText::new("❌").size(14.0)).fill(egui::Color32::TRANSPARENT),
    );
    let max_btn = right_ui.add(
        egui::Button::new(egui::RichText::new("🗖").size(14.0)).fill(egui::Color32::TRANSPARENT),
    );
    let min_btn = right_ui.add(
        egui::Button::new(egui::RichText::new("—").size(14.0)).fill(egui::Color32::TRANSPARENT),
    );

    if close_btn.clicked() {
        ui.send_viewport_cmd(egui::ViewportCommand::Close);
    }
    if max_btn.clicked() {
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
    }
    if min_btn.clicked() {
        ui.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }

    // --- CENTER SECTION: Play and Build Buttons ---
    let mut center_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(header_rect)
            .layout(egui::Layout::top_down(egui::Align::Center)),
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

                let play_text = if app.ui_state.is_playing {
                    "⏹ Stop Game"
                } else {
                    "▶ Play Game"
                };
                let p_res = ui.button(play_text);
                if p_res.clicked() {
                    app.ui_state.is_playing = !app.ui_state.is_playing;
                }
                play_btn_res = Some(p_res);

                let b_res = ui.button("🔨 Build");
                if b_res.clicked() {
                    app.trigger_build(ui.ctx());
                }
                build_btn_res = Some(b_res);
            });
        });
    });

    // --- WINDOW DRAGGING LOGIC ---
    let play_hovered = play_btn_res.map_or(false, |r| r.hovered());
    let build_hovered = build_btn_res.map_or(false, |r| r.hovered());

    let any_btn_hovered = close_btn.hovered()
        || max_btn.hovered()
        || min_btn.hovered()
        || play_hovered
        || build_hovered;

    if ui.rect_contains_pointer(header_rect) && !any_btn_hovered {
        if ui.input(|i| i.pointer.primary_down()) {
            ui.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
    }

    ui.add_space(4.0);
}
