use crate::app::EditorApp;
use eframe::egui;
use en_core::bevy_reflect::{PartialReflect, ReflectKind, ReflectMut};
use en_core::prelude::{AppTypeRegistry, ReflectComponent, ReflectDefault, World};

pub fn setup_inspector_registry(world: &mut World) {
    let registry = world.resource::<AppTypeRegistry>();
    let mut lock = registry.write();

    macro_rules! register_primitive {
        ($($t:ty),+) => {
            $(
                lock.register::<$t>();
            )+
        };
    }

    register_primitive!(
        f32, f64, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, bool, String
    );
}

pub fn draw(ui: &mut egui::Ui, app: &mut EditorApp) {
    let selected_entity = match app.ui_state.selected_entity {
        Some(e) => e,
        None => {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("Select entity").color(en_ui::theme::TEXT_MUTED));
            });
            return;
        }
    };

    if !app.engine.world.get_entity(selected_entity).is_ok() {
        ui.label(egui::RichText::new("Entity no longer exists").color(en_ui::theme::TEXT_MUTED));
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("inspector_scroll")
        .show(ui, |ui| {
            ui.add_space(8.0);

            let registry_arc = app.engine.world.resource::<AppTypeRegistry>().clone();
            let registry = registry_arc.read();

            let mut components = Vec::new();
            if let Ok(entity_ref) = app.engine.world.get_entity(selected_entity) {
                for reg in registry.iter() {
                    if let Some(reflect_comp) = reg.data::<ReflectComponent>() {
                        if reflect_comp.reflect(entity_ref).is_some() {
                            components.push((
                                reg.type_info().type_path_table().short_path().to_string(),
                                reg.type_id(),
                            ));
                        }
                    }
                }
            }

            components.sort_by(|a, b| a.0.cmp(&b.0));

            for (comp_name, type_id) in components {
                let reg = registry.get(type_id).unwrap();
                let reflect_comp = reg.data::<ReflectComponent>().unwrap().clone();

                egui::CollapsingHeader::new(egui::RichText::new(&comp_name).strong())
                    .default_open(true)
                    .show(ui, |ui| {
                        if let Ok(mut entity_mut) = app.engine.world.get_entity_mut(selected_entity)
                        {
                            if let Some(mut reflect_mut) = reflect_comp.reflect_mut(&mut entity_mut)
                            {
                                // Використовуємо PartialReflect для малювання
                                draw_reflect_ui(ui, reflect_mut.as_partial_reflect_mut(), "");
                            }
                        }
                    });
                ui.separator();
            }

            ui.add_space(20.0);
            draw_add_component_menu(ui, app);
        });
}

fn draw_add_component_menu(ui: &mut egui::Ui, app: &mut EditorApp) {
    ui.menu_button(
        egui::RichText::new("➕ Add component").color(en_ui::theme::ACCENT),
        |ui| {
            let registry_arc = app.engine.world.resource::<AppTypeRegistry>().clone();
            let registry = registry_arc.read();

            let mut available: Vec<_> = registry
                .iter()
                .filter(|reg| reg.data::<ReflectComponent>().is_some())
                .map(|reg| {
                    (
                        reg.type_info().type_path_table().short_path().to_string(),
                        reg.type_id(),
                    )
                })
                .collect();

            available.sort_by(|a, b| a.0.cmp(&b.0));

            for (name, type_id) in available {
                if ui.button(&name).clicked() {
                    let reg = registry.get(type_id).unwrap();
                    let reflect_comp = reg.data::<ReflectComponent>().cloned();
                    let reflect_def = reg.data::<ReflectDefault>().cloned();

                    if let (Some(comp), Some(def)) = (reflect_comp, reflect_def) {
                        let mut entity_mut = app
                            .engine
                            .world
                            .entity_mut(app.ui_state.selected_entity.unwrap());
                        comp.insert(&mut entity_mut, &*def.default(), &registry);
                    }
                    ui.close();
                }
            }
        },
    );
}

/// Основна рекурсивна функція для малювання UI. Працює з PartialReflect.
fn draw_reflect_ui(ui: &mut egui::Ui, value: &mut dyn PartialReflect, label: &str) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        if !label.is_empty() {
            ui.label(format!("{}:", label));
        }

        // 1. Спроба обробити як примітив (через спробу даункасту)
        if let Some(v) = value.try_downcast_mut::<f32>() {
            changed |= ui.add(egui::DragValue::new(v).speed(0.1)).changed();
            return;
        }
        if let Some(v) = value.try_downcast_mut::<i32>() {
            changed |= ui.add(egui::DragValue::new(v)).changed();
            return;
        }
        if let Some(v) = value.try_downcast_mut::<bool>() {
            changed |= ui.checkbox(v, "").changed();
            return;
        }
        if let Some(v) = value.try_downcast_mut::<String>() {
            changed |= ui.text_edit_singleline(v).changed();
            return;
        }
        if let Some(v) = value.try_downcast_mut::<glam::Vec2>() {
            ui.label("X");
            changed |= ui.add(egui::DragValue::new(&mut v.x).speed(0.1)).changed();
            ui.label("Y");
            changed |= ui.add(egui::DragValue::new(&mut v.y).speed(0.1)).changed();
            return;
        }
        if let Some(v) = value.try_downcast_mut::<glam::Vec3>() {
            ui.label("X");
            changed |= ui.add(egui::DragValue::new(&mut v.x).speed(0.1)).changed();
            ui.label("Y");
            changed |= ui.add(egui::DragValue::new(&mut v.y).speed(0.1)).changed();
            ui.label("Z");
            changed |= ui.add(egui::DragValue::new(&mut v.z).speed(0.1)).changed();
            return;
        }

        // 2. Якщо не примітив — розбиваємо за структурою
        match value.reflect_kind() {
            ReflectKind::Struct => {
                if let Ok(v) = value.reflect_mut().as_struct() {
                    ui.vertical(|ui| {
                        for i in 0..v.field_len() {
                            let name = v.name_at(i).unwrap_or("?").to_string();
                            let field = v.field_at_mut(i).unwrap();
                            changed |= draw_reflect_ui(ui, field, &name);
                        }
                    });
                }
            }
            ReflectKind::TupleStruct => {
                if let Ok(v) = value.reflect_mut().as_tuple_struct() {
                    for i in 0..v.field_len() {
                        let field = v.field_mut(i).unwrap();
                        changed |= draw_reflect_ui(ui, field, &i.to_string());
                    }
                }
            }
            ReflectKind::List => {
                if let Ok(v) = value.reflect_mut().as_list() {
                    ui.vertical(|ui| {
                        for i in 0..v.len() {
                            let elem = v.get_mut(i).unwrap();
                            changed |= draw_reflect_ui(ui, elem, &format!("[{}]", i));
                        }
                    });
                }
            }
            ReflectKind::Enum => {
                ui.label("(Enum - Requires advanced handling)");
            }
            _ => {
                ui.label(egui::RichText::new("Unsupported type").color(en_ui::theme::TEXT_MUTED));
            }
        }
    });

    changed
}
