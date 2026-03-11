use eframe::egui;

pub fn draw_typed_inspector(
    ui: &mut egui::Ui, 
    value: &mut serde_json::Value, 
    schema: &serde_json::Value 
) -> bool {
    let mut changed = false;

    if let (serde_json::Value::Object(val_map), serde_json::Value::Object(schema_map)) = (value, schema) {
        for (field_name, expected_type_val) in schema_map.iter() {
            let expected_type = expected_type_val.as_str().unwrap_or("");
            
            if let Some(field_value) = val_map.get_mut(field_name) {
                ui.horizontal(|ui| {
                    
                    let is_custom_struct = expected_type == "Color" || expected_type == "Rect" || expected_type == "SpriteId";
                    let is_array = expected_type.starts_with('[');
                    
                    if !is_custom_struct && !is_array && expected_type != "String" {
                        ui.label(field_name);
                    }

                    match expected_type {
                        "f32" | "f64" => {
                            let mut f = field_value.as_f64().unwrap_or(0.0) as f32;
                            if ui.add(egui::DragValue::new(&mut f).speed(0.1)).changed() {
                                *field_value = serde_json::json!(f);
                                changed = true;
                            }
                        },
                        "i8" | "i16" | "i32" | "i64" | "isize" => {
                            let mut i = field_value.as_i64().unwrap_or(0) as i32;
                            if ui.add(egui::DragValue::new(&mut i).speed(1)).changed() {
                                *field_value = serde_json::json!(i);
                                changed = true;
                            }
                        },
                        "u8" | "u16" | "u32" | "u64" | "usize" => {
                            let mut u = field_value.as_u64().unwrap_or(0) as u32;
                            if ui.add(egui::DragValue::new(&mut u).speed(1)).changed() {
                                *field_value = serde_json::json!(u);
                                changed = true;
                            }
                        },
                        "bool" => {
                            let mut b = field_value.as_bool().unwrap_or(false);
                            if ui.checkbox(&mut b, "").changed() {
                                *field_value = serde_json::json!(b);
                                changed = true;
                            }
                        },
                        "String" => {
                            ui.label(field_name);
                            let mut s = field_value.as_str().unwrap_or("").to_string();
                            if ui.text_edit_singleline(&mut s).changed() {
                                *field_value = serde_json::json!(s);
                                changed = true;
                            }
                        },

                        "Color" => {
                            ui.label(field_name);
                            if let Some(obj) = field_value.as_object_mut() {
                                let r = obj.get("r").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                                let g = obj.get("g").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                                let b = obj.get("b").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                                let a = obj.get("a").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;

                                let mut color = egui::Color32::from_rgba_unmultiplied(
                                    (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (a * 255.0) as u8
                                );

                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    obj.insert("r".to_string(), serde_json::json!(color.r() as f32 / 255.0));
                                    obj.insert("g".to_string(), serde_json::json!(color.g() as f32 / 255.0));
                                    obj.insert("b".to_string(), serde_json::json!(color.b() as f32 / 255.0));
                                    obj.insert("a".to_string(), serde_json::json!(color.a() as f32 / 255.0));
                                    changed = true;
                                }
                            }
                        },

                        "Rect" => {
                            ui.label(field_name);
                            if let Some(obj) = field_value.as_object_mut() {
                                let mut x = obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                let mut y = obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                let mut w = obj.get("w").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                let mut h = obj.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 2.0;
                                    
                                    ui.label(egui::RichText::new("X").color(egui::Color32::GRAY));
                                    if ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed() { obj.insert("x".to_string(), serde_json::json!(x)); changed = true; }
                                    
                                    ui.label(egui::RichText::new("Y").color(egui::Color32::GRAY));
                                    if ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed() { obj.insert("y".to_string(), serde_json::json!(y)); changed = true; }
                                    
                                    ui.label(egui::RichText::new("W").color(egui::Color32::GRAY));
                                    if ui.add(egui::DragValue::new(&mut w).speed(0.1)).changed() { obj.insert("w".to_string(), serde_json::json!(w)); changed = true; }
                                    
                                    ui.label(egui::RichText::new("H").color(egui::Color32::GRAY));
                                    if ui.add(egui::DragValue::new(&mut h).speed(0.1)).changed() { obj.insert("h".to_string(), serde_json::json!(h)); changed = true; }
                                });
                            }
                        },

                        "SpriteId" => {
                            ui.label(field_name);
                            let mut id = field_value.as_u64().unwrap_or(0) as u32;
                            
                            ui.horizontal(|ui| {
                                if ui.add(egui::DragValue::new(&mut id)).changed() {
                                    *field_value = serde_json::json!(id);
                                    changed = true;
                                }
                                
                                if ui.button("🖼 Pick").on_hover_text("Open Sprite Picker").clicked() {
                                    println!("[Editor] TODO: Open Sprite picker window");
                                }
                            });
                        },

                        "[f32;2]" => {
                            ui.label(field_name);
                            if let Some(arr) = field_value.as_array_mut() {
                                if arr.len() == 2 {
                                    let mut x = arr[0].as_f64().unwrap_or(0.0) as f32;
                                    let mut y = arr[1].as_f64().unwrap_or(0.0) as f32;
                                    
                                    ui.label(egui::RichText::new("X").color(egui::Color32::LIGHT_RED));
                                    if ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed() { arr[0] = serde_json::json!(x); changed = true; }
                                    
                                    ui.label(egui::RichText::new("Y").color(egui::Color32::LIGHT_GREEN));
                                    if ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed() { arr[1] = serde_json::json!(y); changed = true; }
                                }
                            }
                        },

                        "[f32;3]" => {
                            ui.label(field_name);
                            if let Some(arr) = field_value.as_array_mut() {
                                if arr.len() == 3 {
                                    let mut x = arr[0].as_f64().unwrap_or(0.0) as f32;
                                    let mut y = arr[1].as_f64().unwrap_or(0.0) as f32;
                                    let mut z = arr[2].as_f64().unwrap_or(0.0) as f32;
                                    
                                    ui.label(egui::RichText::new("X").color(egui::Color32::LIGHT_RED));
                                    if ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed() { arr[0] = serde_json::json!(x); changed = true; }
                                    
                                    ui.label(egui::RichText::new("Y").color(egui::Color32::LIGHT_GREEN));
                                    if ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed() { arr[1] = serde_json::json!(y); changed = true; }
                                    
                                    ui.label(egui::RichText::new("Z").color(egui::Color32::LIGHT_BLUE));
                                    if ui.add(egui::DragValue::new(&mut z).speed(0.1)).changed() { arr[2] = serde_json::json!(z); changed = true; }
                                }
                            }
                        },

                        "[f32;4]" => {
                            ui.label(field_name);
                            if let Some(arr) = field_value.as_array_mut() {
                                if arr.len() == 4 {
                                    let mut x = arr[0].as_f64().unwrap_or(0.0) as f32;
                                    let mut y = arr[1].as_f64().unwrap_or(0.0) as f32;
                                    let mut z = arr[2].as_f64().unwrap_or(0.0) as f32;
                                    let mut w = arr[3].as_f64().unwrap_or(0.0) as f32;
                                    
                                    ui.label(egui::RichText::new("X").color(egui::Color32::LIGHT_RED));
                                    if ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed() { arr[0] = serde_json::json!(x); changed = true; }
                                    
                                    ui.label(egui::RichText::new("Y").color(egui::Color32::LIGHT_GREEN));
                                    if ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed() { arr[1] = serde_json::json!(y); changed = true; }
                                    
                                    ui.label(egui::RichText::new("Z").color(egui::Color32::LIGHT_BLUE));
                                    if ui.add(egui::DragValue::new(&mut z).speed(0.1)).changed() { arr[2] = serde_json::json!(z); changed = true; }

                                    ui.label(egui::RichText::new("W").color(egui::Color32::GOLD));
                                    if ui.add(egui::DragValue::new(&mut w).speed(0.1)).changed() { arr[3] = serde_json::json!(w); changed = true; }
                                }
                            }
                        },

                        _ => {
                            ui.label(field_name);
                            ui.label(egui::RichText::new(format!("(Unsupported type: {})", expected_type)).color(egui::Color32::RED)); 
                        }
                    }
                });
            }
        }
    }
    
    changed
}