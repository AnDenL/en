use eframe::egui;
use en_core::prelude::{AppTypeRegistry, ReflectComponent, ReflectDefault, World};
use crate::app::EditorApp;
use bevy_inspector_egui::bevy_inspector::ui_for_entity;

pub fn setup_inspector_registry(world: &mut World) {
    let registry = world.resource::<AppTypeRegistry>();
    let mut lock = registry.write();

    macro_rules! register_primitive {
        ($($t:ty),+) => {
            $(
                lock.register::<$t>();
                lock.register_type_data::<$t, bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl>();
            )+
        };
    }

    register_primitive!(
        f32, f64,
        i8, i16, i32, i64, isize,
        u8, u16, u32, u64, usize,
        bool, String
    );

    lock.register::<en_core::types::Color>();
    lock.register::<en_core::types::SpriteId>();
    lock.register::<en_core::types::Rect>();
    lock.register::<en_core::components::Transform>();
    lock.register::<en_core::components::Render>();
}

pub fn draw(ui: &mut egui::Ui, app: &mut EditorApp) {
    ui.heading(egui::RichText::new("⚙ Inspector").color(en_ui::theme::ACCENT));
    ui.separator();

    if let Some(selected_entity) = app.ui_state.selected_entity {
        
        if !app.engine.world.get_entity(selected_entity).is_ok() {
            ui.label(egui::RichText::new("Entity no longer exists").color(en_ui::theme::TEXT_MUTED));
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("inspector_scroll")
            .show(ui, |ui| {
                ui.add_space(8.0);
                ui_for_entity(&mut app.engine.world, selected_entity, ui);
        });

        ui.add_space(15.0);
        ui.separator();

        ui.menu_button(egui::RichText::new("➕ Add Component").color(en_ui::theme::TEXT_MAIN), |ui| {
            
            let type_registry_arc = app.engine.world.resource::<AppTypeRegistry>().clone();
            
            let components: Vec<(String, std::any::TypeId)> = {
                let registry = type_registry_arc.read();
                let mut comps: Vec<_> = registry.iter()
                    .filter_map(|reg| {
                        if reg.data::<ReflectComponent>().is_some() {
                            let short_name = reg.type_info().type_path_table().short_path().to_string();
                            Some((short_name, reg.type_id()))
                        } else {
                            None
                        }
                    })
                    .collect();
                
                comps.sort_by(|a, b| a.0.cmp(&b.0));
                comps
            };

            for (name, type_id) in components {
                if ui.button(&name).clicked() {
                    
                    let (reflect_component, reflect_default) = {
                        let registry = type_registry_arc.read();
                        let reg = registry.get(type_id).unwrap();
                        (
                            reg.data::<ReflectComponent>().cloned(),
                            reg.data::<ReflectDefault>().cloned()
                        )
                    };

                    let mut entity_mut = app.engine.world.entity_mut(selected_entity);

                    if let (Some(comp), Some(def)) = (&reflect_component, &reflect_default) {
                        let default_val = def.default();
                        comp.insert(
                            &mut entity_mut, 
                            &*default_val, 
                            &type_registry_arc.read() 
                        );
                    } 
                    else if let Some(inserter) = app.engine.inserters.get(&name) {
                        (inserter)(&mut entity_mut, serde_json::json!({}));
                    } else {
                        println!("[Editor] Cannot instantiate component: {}", name);
                    }

                    ui.close(); 
                }
            }
        });

    } else {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("No entity selected").color(en_ui::theme::TEXT_MUTED));
        });
    }
}