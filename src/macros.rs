#[macro_export]
macro_rules! define_all_components {
    ( 
        $( 
            $(#[$s_meta:meta])* $name:ident { 
                $( $(#[$f_meta:meta])* $field:ident: $ftype:ty $( = $default:expr )? ),* $(,)? 
            } 
        ),* $(,)? 
    ) => {
        $(
            $(#[$s_meta])*
            #[derive(Clone, serde::Serialize, serde::Deserialize)]
            #[serde(default)]
            #[allow(non_snake_case)]
            pub struct $name {
                $( 
                    $(#[$f_meta])*
                    #[serde(default)]
                    pub $field: $ftype 
                ),*
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        $( 
                            $field: $crate::define_all_components!(@default_val $($default)?), 
                        )*
                    }
                }
            }
        )*

        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(default)] 
        #[allow(non_snake_case)]
        pub struct Scene {
            $( 
                #[serde(default)]
                pub $name: Vec<(u64, $name)> 
            ),*
        }
        impl Default for Scene {
            fn default() -> Self {
                Self {
                    $( $name: Vec::new() ),*
                }
            }
        }

        pub fn save_scene(world: &mut hecs::World) -> Vec<u8> {
            let scene = Scene {
                $(
                    $name: world.query_mut::<&$name>()
                        .into_iter()
                        .map(|(entity, comp)| (entity.to_bits().get(), comp.clone()))
                        .collect() 
                ),*
            };
            
            rmp_serde::to_vec_named(&scene).expect("Failed to serialize scene")
        }

        pub fn load_scene(world: &mut hecs::World, data: &[u8]) {
            world.clear();
            
            let scene: Scene = rmp_serde::from_slice(data).unwrap_or_else(|err| {
                println!("cargo:warning=Failed to load scene: {}", err);
                Scene::default()
            });

            let mut id_map = std::collections::HashMap::new();

            $(
                for (old_id, comp) in scene.$name {
                    let new_entity = *id_map.entry(old_id).or_insert_with(|| world.spawn(()));
                    world.insert_one(new_entity, comp).unwrap();
                }
            )*
        }

        pub fn duplicate_entity(world: &mut hecs::World, entity: hecs::Entity) -> hecs::Entity {
            let mut builder = hecs::EntityBuilder::new();
            {
                if let Ok(entity_ref) = world.entity(entity) {
                    $(
                        if let Some(comp) = entity_ref.get::<&$name>() {
                            builder.add((*comp).clone());
                        }
                    )*
                }
            }
            world.spawn(builder.build())
        }

        #[cfg(debug_assertions)]
        pub fn draw_component_menu(ui: &mut egui_macroquad::egui::Ui, cmd: &mut hecs::CommandBuffer, entity: hecs::Entity) {
            ui.menu_button("➕ Add component", |ui| {
                $(
                    if ui.button(stringify!($name)).clicked() {
                        cmd.insert_one(entity, $name::default());
                        ui.close_menu();
                    }
                )*
            });
        }

        #[cfg(debug_assertions)]
        pub fn draw_entity_inspector(ui: &mut egui_macroquad::egui::Ui, world: &hecs::World, cmd: &mut hecs::CommandBuffer, entity: hecs::Entity, sprite_manager: &crate::sprite_manager::SpriteManager) {
            $(
                if let Ok(mut comp) = world.get::<&mut $name>(entity) {
                    let id = ui.make_persistent_id(stringify!($name));
                    let mut remove_clicked = false;

                    egui_macroquad::egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                    .show_header(ui, |ui| {
                        ui.strong(stringify!($name));
                        ui.with_layout(egui_macroquad::egui::Layout::right_to_left(egui_macroquad::egui::Align::Center), |ui| {
                            if ui.button("❌").on_hover_text("Remove").clicked() {
                                remove_clicked = true;
                            }
                        });
                    })
                    .body(|ui| {
                    egui_macroquad::egui::Grid::new(stringify!($name))
                    .num_columns(2)
                    .spacing([40.0, 4.0]) 
                    .striped(true)
                    .show(ui, |ui| {
                            $(
                    ui.label(stringify!($field));
                    {
                        if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<f32>() {
                            ui.add_sized(
                                ui.available_size(),
                                egui_macroquad::egui::DragValue::new(val).speed(0.1)
                            );
                        } else if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<f64>() {
                            ui.add_sized(
                                ui.available_size(),
                                egui_macroquad::egui::DragValue::new(val).speed(0.1)
                            );
                        } else if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<i32>() {
                            ui.add_sized(
                                ui.available_size(),
                                egui_macroquad::egui::DragValue::new(val).speed(0.1)
                            );
                        } else if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<u32>() {
                            ui.add_sized(
                                ui.available_size(),
                                egui_macroquad::egui::DragValue::new(val).speed(0.1)
                            );
                        } else if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<usize>() {
                            ui.add_sized(
                                ui.available_size(),
                                egui_macroquad::egui::DragValue::new(val).speed(0.1)
                            );
                        } else if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<i64>() {
                            ui.add_sized(
                                ui.available_size(),
                                egui_macroquad::egui::DragValue::new(val).speed(0.1)
                            );
                        } else if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<u64>() {
                            ui.add_sized(
                                ui.available_size(),
                                egui_macroquad::egui::DragValue::new(val).speed(0.1)
                            );
                        } else if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<String>() {
                            ui.add_sized(
                                ui.available_size(),
                                egui_macroquad::egui::TextEdit::singleline(val)
                            );
                        } else if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<[f32; 4]>() {
                            let mut c = *val;
                            if ui.color_edit_button_rgba_unmultiplied(&mut c).changed() {
                                *val = c;
                            }
                        } else if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<[f32; 2]>() {
                            ui.horizontal(|ui| {
                                ui.add(egui_macroquad::egui::DragValue::new(&mut val[0]).speed(0.1).prefix("X: "));
                                ui.add(egui_macroquad::egui::DragValue::new(&mut val[1]).speed(0.1).prefix("Y: "));
                            });
                        } else if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<bool>() {
                            ui.checkbox(val, "");
                        } else {
                            let mut sprite_changed = false;

                            if let Some(val) = (&mut comp.$field as &mut dyn std::any::Any).downcast_mut::<crate::sprite_manager::SpriteId>() {
                                ui.horizontal(|ui| {
                                ui.label("🖼");
                                let current_name = sprite_manager.sprite_names.get(&val.0)
                                    .cloned()
                                    .unwrap_or_else(|| "Unknown".to_string());
                                
                                let picker_id = ui.make_persistent_id(format!("{}_{}_picker", stringify!($name), stringify!($field)));
                                let search_id = picker_id.with("search");

                                let mut is_open = ui.data_mut(|d| d.get_temp::<bool>(picker_id).unwrap_or(false));

                                if ui.button(format!("{} ⏷", current_name)).clicked() {
                                    is_open = true;
                                }

                                if is_open {
                                    let mut open = is_open;
                                    let mut should_close = false; 

                                    egui_macroquad::egui::Window::new(format!("Select Sprite"))
                                    .open(&mut open)
                                    .default_size([450.0, 350.0])
                                    .collapsible(true)
                                    .vscroll(false)
                                    .show(ui.ctx(), |ui| {
                                        
                                        let mut search_text = ui.data_mut(|d| d.get_temp::<String>(search_id).unwrap_or_default());
                                        ui.horizontal(|ui| {
                                            ui.label("🔍");
                                            ui.text_edit_singleline(&mut search_text);
                                            if ui.button("✖").clicked() { search_text.clear(); }
                                        });
                                        ui.separator();

                                        let mut names: Vec<_> = sprite_manager.sprite_names.iter().collect();
                                        names.sort_by(|a, b| a.1.cmp(b.1));

                                        egui_macroquad::egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                                        ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing = egui_macroquad::egui::vec2(10.0, 10.0);
                                        for (hash_id, name) in names {
                                            if search_text.is_empty() || name.to_lowercase().contains(&search_text.to_lowercase()) {
                                                ui.allocate_ui(egui_macroquad::egui::vec2(110.0, 130.0), |ui| {
                                                ui.vertical_centered(|ui| {
                                                    
                                                if let Some(sprite) = sprite_manager.sprites.get(hash_id) {
                                                    let miniquad_id = sprite.texture.raw_miniquad_id();

                                                    let raw_handle_u32 = unsafe {
                                                        let internal_gl = get_internal_gl();
                                                        let raw_id = internal_gl.quad_context.texture_raw_id(miniquad_id);
                                                        std::mem::transmute::<_, u32>(raw_id)
                                                    };
                                                    let egui_texture_id = egui_macroquad::egui::TextureId::User(raw_handle_u32 as u64);

                                                    let tw = sprite.texture.width();
                                                    let th = sprite.texture.height();
                                                    
                                                    let uv = egui_macroquad::egui::Rect::from_min_max(
                                                        egui_macroquad::egui::pos2(sprite.source_rect.x / tw, sprite.source_rect.y / th),
                                                        egui_macroquad::egui::pos2((sprite.source_rect.x + sprite.source_rect.w) / tw, (sprite.source_rect.y + sprite.source_rect.h) / th),
                                                    );

                                                    let (rect, response) = ui.allocate_exact_size(egui_macroquad::egui::vec2(100.0, 100.0), egui_macroquad::egui::Sense::click());
                                                
                                                    ui.painter().image(egui_texture_id, rect, uv, egui_macroquad::egui::Color32::WHITE);
                                                    
                                                    if response.hovered() {
                                                        ui.painter().rect_stroke(rect, 0.0, (2.0, egui_macroquad::egui::Color32::WHITE), egui_macroquad::egui::StrokeKind::Inside);
                                                    }
                                                    
                                                    let is_selected = val.0 == *hash_id;
                                                    if is_selected {
                                                        ui.painter().rect_stroke(rect, 0.0, (3.0, egui_macroquad::egui::Color32::GREEN), egui_macroquad::egui::StrokeKind::Inside);
                                                    }

                                                    if response.clicked() {
                                                        val.0 = *hash_id;
                                                        sprite_changed = true;
                                                        should_close = true;
                                                    }
                                                }
                                                
                                                ui.add(egui_macroquad::egui::Label::new(
                                                    egui_macroquad::egui::RichText::new(name).size(12.0)
                                                ).truncate());
                                                });
                                                });
                                            }
                                        }
                                        });
                                        });
                                        
                                        ui.data_mut(|d| d.insert_temp(search_id, search_text));
                                        });
                                        if should_close {
                                            open = false;
                                        }
                                        ui.data_mut(|d| d.insert_temp(picker_id, open));
                                    }
                                });
                                }

                            if sprite_changed {
                                if let Some(render_comp) = (&mut *comp as &mut dyn std::any::Any).downcast_mut::<Render>() {
                                    render_comp.cached_sprite = None;
                                }
                            }
                            }
                            }
                            ui.end_row();
                        )*
                        });
                    });
                    ui.add_space(4.0); 
                    if remove_clicked {
                        cmd.remove_one::<$name>(entity);
                    }
                }
            )*
        }
    };
    (@default_val $val:expr) => { $val };
    (@default_val) => { Default::default() };
}