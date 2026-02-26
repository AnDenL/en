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
                                            let current_name = sprite_manager.sprite_names.get(val.0).cloned().unwrap_or_else(|| "Unknown".to_string());
                                            
                                            egui_macroquad::egui::ComboBox::from_id_salt(format!("{}_{}", stringify!($name), stringify!($field)))
                                            .selected_text(&current_name)
                                            .show_ui(ui, |ui| {
                                                for (id, name) in sprite_manager.sprite_names.iter().enumerate() {
                                                    if ui.selectable_value(&mut val.0, id, name).changed() {
                                                        sprite_changed = true;
                                                    }
                                                }
                                            });
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