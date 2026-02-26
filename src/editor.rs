use hecs::{World, CommandBuffer, Entity};
use macroquad::prelude::*;
use egui_macroquad::egui;
use crate::components::*;

#[cfg(debug_assertions)]
pub fn handle_editor_input(
    world: &mut World, camera: &Camera2D, 
    selected: &mut Option<hecs::Entity>, dragging: &mut Option<hecs::Entity>, 
    offset: &mut Vec2, ctx_world: &mut Option<Vec2>, ctx_screen: &mut Option<Vec2>,
    brush_mode: &mut bool 
) {
    let m_pos = mouse_position();
    let mouse_world = camera.screen_to_world(vec2(m_pos.0, m_pos.1));

    if is_key_pressed(KeyCode::B) {
        *brush_mode = !*brush_mode;
        if *brush_mode { println!("Brush Mode: ON"); } 
        else { println!("Brush Mode: OFF"); }
    }

    if *brush_mode {
        if is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right) {
            if let Some(entity) = selected {
                if let Ok(mut tm) = world.get::<&mut TileMap>(*entity) {
                    if let Ok(pos) = world.get::<&Pos>(*entity) {
                        let offset_x = (tm.width as f32 * tm.tile_size) / 2.0;
                        let offset_y = (tm.height as f32 * tm.tile_size) / 2.0;
                        let map_start_x = pos.x - offset_x;
                        let map_start_y = pos.y - offset_y;

                        let local_x = mouse_world.x - map_start_x;
                        let local_y = mouse_world.y - map_start_y;
                        
                        let grid_x = (local_x / tm.tile_size).floor() as i32;
                        let grid_y = (local_y / tm.tile_size).floor() as i32;

                        if grid_x >= 0 && grid_x < tm.width as i32 && grid_y >= 0 && grid_y < tm.height as i32 {
                            let idx = (grid_y as usize) * tm.width + (grid_x as usize);
                            if is_mouse_button_down(MouseButton::Left) {
                                tm.tiles[idx] = tm.brush_sprite.0;
                            } else {
                                tm.tiles[idx] = 0; 
                            }
                        }
                    }
                }
            }
        }
        return;
    }

    let m_pos = mouse_position();
    let mouse_world = camera.screen_to_world(vec2(m_pos.0, m_pos.1));

    if is_mouse_button_pressed(MouseButton::Left) {
        *ctx_world = None; 
        *ctx_screen = None;

        let mut clicked = None;
        for (id, (pos, ren)) in world.query_mut::<(&Pos, &Render)>() {
            let rect = Rect::new(pos.x - ren.w / 2.0, pos.y - ren.h / 2.0, ren.w, ren.h);
            if rect.contains(mouse_world) {
                clicked = Some(id);
                *offset = vec2(pos.x, pos.y) - mouse_world;
            }
        }
        *selected = clicked; 
        *dragging = clicked; 
    }

    if is_mouse_button_down(MouseButton::Left) {
        if let Some(entity) = dragging {
            if let Ok(mut pos) = world.get::<&mut Pos>(*entity) {
                let target_x = mouse_world.x + offset.x;
                let target_y = mouse_world.y + offset.y;

                if is_key_down(KeyCode::LeftShift) {
                    pos.x = (target_x / 16.0).round() * 16.0;
                    pos.y = (target_y / 16.0).round() * 16.0;
                } else {
                    pos.x = target_x;
                    pos.y = target_y;
                }
            }
        }
    }

    if is_mouse_button_released(MouseButton::Left) { *dragging = None; }
    if is_mouse_button_pressed(MouseButton::Right) {
        *ctx_world = Some(mouse_world);
        *ctx_screen = Some(vec2(m_pos.0, m_pos.1));
    }
}

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
                    if ui.button("■ Spawn Square").clicked() {
                        let new_ent = world.spawn((
                            Pos { x: world_pos.x, y: world_pos.y },
                            Render::default(), 
                        ));
                        *selected_entity = Some(new_ent);
                        *ctx_menu_world = None;
                        *ctx_menu_screen = None;
                    }
                    if ui.button("⛶ Spawn Tilemap").clicked() {
                        let new_ent = world.spawn((
                            Pos { x: world_pos.x, y: world_pos.y },
                            TileMap::default(), 
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