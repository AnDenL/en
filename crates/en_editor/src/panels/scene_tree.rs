use eframe::egui;
use en_core::{bevy_ecs::entity::Entity};
use crate::app::EditorApp;

/// Draws the Scene Tree Tab (Hierarchy of all entities in the ECS World)
pub fn draw(ui: &mut egui::Ui, app: &mut EditorApp) {
    // --- TOP BAR OF THE SCENE TREE ---
    ui.horizontal(|ui| {
        ui.heading(egui::RichText::new("Hierarchy").color(en_ui::theme::ACCENT));
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("➕ Add Entity").clicked() {
                // Spawn a new empty entity directly into the Engine's ECS!
                let new_entity = app.engine.world.spawn_empty().id();
                app.ui_state.selected_entity = Some(new_entity);
                app.log(&format!("Spawned new entity: {:?}", new_entity));
            }
        });
    });
    
    ui.separator();

    // Variables to store actions that will modify the ECS world.
    // We do this OUTSIDE the UI loop to avoid borrow checker errors!
    let mut action_delete: Option<Entity> = None;
    let mut action_duplicate: Option<Entity> = None;

    // --- QUERY ENTITIES ---
    // We ask the ECS for all existing entities.
    let mut query = app.engine.world.query::<Entity>();
    
    // Collect them into a Vec so we don't hold a reference to the world while drawing UI
    let entities: Vec<Entity> = query.iter(&app.engine.world).collect();

    // --- DRAW ENTITY LIST ---
    egui::ScrollArea::vertical().show(ui, |ui| {
        let mut query_name = app.engine.world.query::<&en_core::components::Name>();
        for entity in entities {
            let is_selected = app.ui_state.selected_entity == Some(entity);
            
            // TODO: In the next step (Inspector), we will add a real `Name` component.
            // For now, we just use the internal ECS ID.
            let entity_display_name = if let Ok(name_comp) = query_name.get(&app.engine.world, entity) {
                name_comp.0.clone()
            } else {
                format!("Entity {}", entity.index())
            };

            // Draw the selectable label
            let response = ui.selectable_label(is_selected, entity_display_name);

            // Handle Left Click (Selection)
            if response.clicked() {
                app.ui_state.selected_entity = Some(entity);
            }

            // --- CONTEXT MENU (RIGHT CLICK) ---
            response.context_menu(|ui| {
                if ui.button("📋 Duplicate").clicked() {
                    action_duplicate = Some(entity);
                    ui.close();
                }
                
                if ui.button("✏ Rename").clicked() {
                    // We will implement this when we add the `Name` component!
                    app.log("Renaming requires a Name component (coming in the Inspector update!)");
                    ui.close();
                }
                
                ui.separator();
                
                if ui.button(egui::RichText::new("🗑 Delete").color(en_ui::theme::ERROR)).clicked() {
                    action_delete = Some(entity);
                    ui.close();
                }
            });
        }
    });

    // --- EXECUTE ACTIONS SAFELY ---
    // Now that the UI loop is over, we can safely mutate the ECS world!

    // Handle Deletion
    if let Some(entity_to_delete) = action_delete {
        app.engine.world.despawn(entity_to_delete);
        
        // If we deleted the currently selected entity, clear the selection
        if app.ui_state.selected_entity == Some(entity_to_delete) {
            app.ui_state.selected_entity = None;
        }
        
        app.log(&format!("Deleted Entity {}", entity_to_delete.index()));
    }

    // Handle Duplication
    if let Some(entity_to_duplicate) = action_duplicate {
        // Full duplication requires reflection (copying all components).
        // For now, we just spawn a new empty entity to prove the UI works.
        let new_entity = app.engine.world.spawn_empty().id();
        app.ui_state.selected_entity = Some(new_entity);
        
        app.log(&format!(
            "Duplicated Entity {} (TODO: Copy components with bevy_reflect)", 
            entity_to_duplicate.index()
        ));
    }
}