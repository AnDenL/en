use eframe::egui;
use en_core::{bevy_ecs::entity::Entity, bevy_reflect::serde::TypedReflectSerializer};
use crate::app::EditorApp;
use en_core::components::{Name, Transform, SpriteRenderer};

pub fn draw(ui: &mut egui::Ui, app: &mut EditorApp) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui::RichText::new("💾 Save Scene").color(en_ui::theme::SUCCESS)).clicked() {
                
                let registry_arc = app.engine.world.resource::<en_core::bevy_ecs::reflect::AppTypeRegistry>().clone();
                let registry = registry_arc.read();

                let mut scene_entities = Vec::new();
                let mut query = app.engine.world.query::<Entity>();
                let entities: Vec<Entity> = query.iter(&app.engine.world).collect();

                for entity in entities {
                    let entity_ref = app.engine.world.entity(entity);
                    let mut components_map = std::collections::HashMap::new();
                    
                    let mut entity_name = format!("Entity {}", entity.index());
                    if let Some(name_comp) = entity_ref.get::<Name>() {
                        entity_name = name_comp.0.clone();
                    }

                    for reg in registry.iter() {
                        if let Some(reflect_comp) = reg.data::<en_core::bevy_ecs::reflect::ReflectComponent>() {
                            if let Some(reflected) = reflect_comp.reflect(entity_ref) {
                                let short_name = reg.type_info().type_path_table().short_path().to_string();
                                
                                let serializer = TypedReflectSerializer::new(reflected, &registry);
                                if let Ok(json_value) = serde_json::to_value(&serializer) {
                                    components_map.insert(short_name, json_value);
                                } else {
                                    eprintln!("[Editor] Failed to serialize component: {}", short_name);
                                }
                            }
                        }
                    }

                    scene_entities.push(en_core::scene::EntityData {
                        name: entity_name,
                        components: components_map,
                    });
                }

                let scene = en_core::scene::Scene { entities: scene_entities };
                let scene_path = format!("{}/main.scene", app.ui_state.project_path);
                scene.save(&scene_path);
                app.log(&format!("Scene successfully saved to {}", scene_path));
            }
        });
    });
    
    ui.separator();

    let mut action_delete: Option<Entity> = None;
    let mut action_duplicate: Option<Entity> = None;
    let mut action_rename: Option<(Entity, String)> = None;
    let mut action_spawn_empty = false;
    let mut action_spawn_square = false;

    let rename_state_id = egui::Id::new("renaming_entity_id");
    let rename_buf_id = egui::Id::new("renaming_buffer_id");
    
    let mut renaming_entity: Option<Entity> = ui.data_mut(|d| d.get_temp(rename_state_id).unwrap_or_default());
    let mut rename_buffer: String = ui.data_mut(|d| d.get_temp(rename_buf_id).unwrap_or_default());

    let mut query = app.engine.world.query::<(Entity, Option<&Name>)>();
    let entities: Vec<(Entity, Option<String>)> = query.iter(&app.engine.world)
        .map(|(e, n)| (e, n.map(|name| name.0.clone())))
        .collect();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (entity, name_opt) in entities {
            let is_selected = app.ui_state.selected_entity == Some(entity);
            let display_name = name_opt.unwrap_or_else(|| format!("Entity {}", entity.index()));

            if renaming_entity == Some(entity) {
                let response = ui.text_edit_singleline(&mut rename_buffer);
                response.request_focus();
                
                if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    action_rename = Some((entity, rename_buffer.clone()));
                    renaming_entity = None; 
                }
            } 
            else {
                let response = ui.selectable_label(is_selected, &display_name);

                if response.clicked() {
                    app.ui_state.selected_entity = Some(entity);
                }

                response.context_menu(|ui| {
                    if ui.button("✏ Rename").clicked() {
                        renaming_entity = Some(entity);
                        rename_buffer = display_name.clone();
                        ui.close();
                    }
                    if ui.button("📋 Duplicate").clicked() {
                        action_duplicate = Some(entity);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(egui::RichText::new("🗑 Delete").color(en_ui::theme::ERROR)).clicked() {
                        action_delete = Some(entity);
                        ui.close();
                    }
                });
            }
        }
        let remaining_space = ui.available_size();
        if remaining_space.y > 0.0 {
            let (_rect, response) = ui.allocate_exact_size(remaining_space, egui::Sense::click());
            
            if response.clicked() {
                app.ui_state.selected_entity = None;
            }

            response.context_menu(|ui| {
                ui.menu_button("➕ Create Entity...", |ui| {
                    if ui.button("📄 Empty").clicked() {
                        action_spawn_empty = true;
                        ui.close();
                    }
                    if ui.button("⏹ Square (Sprite)").clicked() {
                        action_spawn_square = true;
                        ui.close();
                    }
                });
            });
        }
    });

    ui.data_mut(|d| d.insert_temp(rename_state_id, renaming_entity));
    ui.data_mut(|d| d.insert_temp(rename_buf_id, rename_buffer));

    // 1. Rename
    if let Some((entity, new_name)) = action_rename {
        if let Ok(mut ent_mut) = app.engine.world.get_entity_mut(entity) {
            ent_mut.insert(Name(new_name));
        }
    }

    // 2. Spawn Empty
    if action_spawn_empty {
        let new_ent = app.engine.world.spawn_empty().id();
        app.engine.world.entity_mut(new_ent).insert(Name("New Entity".to_string()));
        app.ui_state.selected_entity = Some(new_ent);
    }

    // 3. Spawn Square Template
    if action_spawn_square {
        let mut ent_mut = app.engine.world.spawn_empty();
        let new_ent = ent_mut.id();
        ent_mut.insert(Name("Square".to_string()));
        ent_mut.insert(Transform::default());
        ent_mut.insert(SpriteRenderer::default());
        app.ui_state.selected_entity = Some(new_ent);
    }

    // 4. Delete
    if let Some(entity_to_delete) = action_delete {
        app.engine.world.despawn(entity_to_delete);
        if app.ui_state.selected_entity == Some(entity_to_delete) {
            app.ui_state.selected_entity = None;
        }
    }

    // 5. Duplicate
    if let Some(entity_to_duplicate) = action_duplicate {
        let registry_arc = app.engine.world.resource::<en_core::bevy_ecs::reflect::AppTypeRegistry>().clone();
        let registry = registry_arc.read();

        let mut cloned_components = Vec::new();
        {
            let entity_ref = app.engine.world.entity(entity_to_duplicate);
            for reg in registry.iter() {
                if let Some(reflect_comp) = reg.data::<en_core::bevy_ecs::reflect::ReflectComponent>() {
                    if let Some(reflected) = reflect_comp.reflect(entity_ref) {
                        if let Ok(cloned_val) = reflected.reflect_clone() {
                            cloned_components.push((reflect_comp.clone(), cloned_val));
                        }
                    }
                }
            }
        }

        let mut new_ent_mut = app.engine.world.spawn_empty();
        for (reflect_comp, reflected_val) in cloned_components {
            reflect_comp.insert(&mut new_ent_mut, reflected_val.as_partial_reflect(), &registry); 
        }

        if let Some(mut name_comp) = new_ent_mut.get_mut::<Name>() {
            name_comp.0 = format!("{} (Copy)", name_comp.0);
        } else {
            new_ent_mut.insert(Name("Copy".to_string()));
        }

        let new_ent = new_ent_mut.id();
        app.ui_state.selected_entity = Some(new_ent);
        app.log(&format!("Duplicated Entity {}", entity_to_duplicate.index()));
    }
}