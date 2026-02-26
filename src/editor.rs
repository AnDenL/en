use hecs::{World, CommandBuffer, Entity};
use egui_macroquad::egui;
use crate::components::*;

#[cfg(debug_assertions)]
pub fn draw_editor(
    world: &mut World, 
    selected_entity: &mut Option<Entity>, 
    is_paused: &mut bool, 
    block_input: &mut bool,
    ctx_menu_world: &mut Option<macroquad::math::Vec2>,
    ctx_menu_screen: &mut Option<macroquad::math::Vec2>,
    sprite_manager: &crate::sprite_manager::SpriteManager 
) {
    let mut cmd = CommandBuffer::new();
    egui_macroquad::ui(|egui_ctx| {
        *block_input = egui_ctx.wants_pointer_input() || egui_ctx.is_pointer_over_area();

        if let (Some(world_pos), Some(screen_pos)) = (*ctx_menu_world, *ctx_menu_screen) {
            egui::Area::new("scene_context_menu".into())
            .fixed_pos(egui::pos2(screen_pos.x, screen_pos.y)) 
            .order(egui::Order::Tooltip) 
            .show(egui_ctx, |ui| {
                egui::Frame::menu(ui.style()).show(ui, |ui| {

                    if let Some(entity) = *selected_entity {
                        if ui.button("🗑 Delete entity").clicked() {
                            cmd.despawn(entity);
                            *selected_entity = None;
                        }
                        if ui.button("🗐 Copy entity").clicked() {
                            let new_entity = duplicate_entity(world, entity);
                            *selected_entity = Some(new_entity);
                        }
                        ui.separator();
                    }

                    if ui.button("Spawn Empty").clicked() {
                        let new_ent = world.spawn((Pos { x: world_pos.x, y: world_pos.y },)); 
                        *selected_entity = Some(new_ent);
                        *ctx_menu_world = None;
                        *ctx_menu_screen = None;
                    }
                    if ui.button("🟩 Spawn Square").clicked() {
                        let new_ent = world.spawn((
                            Pos { x: world_pos.x, y: world_pos.y },
                            Render::default(), 
                        ));
                        *selected_entity = Some(new_ent);
                        *ctx_menu_world = None;
                        *ctx_menu_screen = None;
                    }
                });
            });
        }


        egui::TopBottomPanel::top("top_bar").show(egui_ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.heading("🛠 En Editor");
                
                ui.separator();

                let label = if *is_paused { "▶ Play" } else { "⏸ Pause" };
                if ui.selectable_label(*is_paused, label).clicked() {
                    *is_paused = !*is_paused;
                }

                ui.separator();

                if ui.button("💾 Save").clicked() {
                    let bytes = save_scene(world); 
                    if let Err(e) = std::fs::write("Scene.bin", bytes) {
                        println!("cargo:warning=Failed to write file: {}", e);
                    }
                }

                ui.add_space(4.0);

                if ui.button("📂 Load").clicked() {
                    if let Ok(bytes) = std::fs::read("Scene.bin") {
                        load_scene(world, &bytes); 
                        *selected_entity = None;
                        
                        for (_id, ren) in world.query_mut::<&mut Render>() {
                            ren.cached_sprite = None;
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("FPS: {:.0}", macroquad::time::get_fps()));
                });
            });
        });

        egui::Window::new("🌍 Hierarchy")
            .default_size([200.0, 400.0]) 
            .vscroll(true) 
            .show(egui_ctx, |ui| {
                if ui.button("✨ Spawn Entity").clicked() {
                    let new_ent = world.spawn(());
                    *selected_entity = Some(new_ent);
                }
                
                ui.separator();

                let entities: Vec<Entity> = world.iter().map(|entity_ref| entity_ref.entity()).collect();
                for entity in entities {
                    ui.horizontal(|ui| {
                        let is_selected = *selected_entity == Some(entity);
                        if ui.selectable_label(is_selected, format!("ID: {:?}", entity.id())).clicked() {
                            *selected_entity = Some(entity);
                        }   

                        if ui.button("🗐").on_hover_text("Duplicate").clicked() {
                            let new_entity = duplicate_entity(world, entity);
                            *selected_entity = Some(new_entity);
                        }

                        if ui.button("🗑").clicked() {
                            cmd.despawn(entity);
                            if is_selected { *selected_entity = None; }
                        }
                    });
                }
            });

        egui::Window::new("🛠 Inspector")
            .default_size([250.0, 400.0])
            .vscroll(true)
            .show(egui_ctx, |ui| {
                if let Some(entity) = *selected_entity {
                    if world.contains(entity) {
                        draw_component_menu(ui, &mut cmd, entity);
                        ui.separator();
                        draw_entity_inspector(ui, world, &mut cmd, entity, sprite_manager);
                    } else {
                        *selected_entity = None;
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select an entity to inspect");
                    });
                }
            });
    });

    cmd.run_on(world);
}