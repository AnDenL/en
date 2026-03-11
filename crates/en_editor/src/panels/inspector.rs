use eframe::egui;
use crate::app::EditorApp;

// We use the magic function from bevy_inspector_egui
use bevy_inspector_egui::bevy_inspector::ui_for_entity;

/// Draws the Inspector Panel using automatic Reflection
pub fn draw(ui: &mut egui::Ui, app: &mut EditorApp) {
    ui.heading(egui::RichText::new("Inspector").color(en_ui::theme::ACCENT));
    ui.separator();

    // 1. Check if an entity is selected in the Scene Tree
    if let Some(selected_entity) = app.ui_state.selected_entity {
        
        // Check if the entity still exists in the world
        if !app.engine.world.get_entity(selected_entity).is_ok() {
            ui.label(egui::RichText::new("Entity no longer exists").color(en_ui::theme::TEXT_MUTED));
            return;
        }
        
        // This single function looks into the ECS world, finds all components attached to the entity,
        // looks up their types in the AppTypeRegistry, and draws sliders, colors, and checkboxes for them!
        ui_for_entity(&mut app.engine.world, selected_entity, ui);

        ui.add_space(15.0);

        // 3. Add Component Button
        ui.menu_button("➕ Add Component", |ui| {
            // Here you will iterate over available components and insert them.
            // For now, it's a stub until we rebuild the inserter logic for pure ECS.
            if ui.button("Transform").clicked() {
                app.log("TODO: Insert Transform using generic reflection inserter");
                ui.close();
            }
        });

    } else {
        // If nothing is selected
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("No entity selected").color(en_ui::theme::TEXT_MUTED));
        });
    }
}