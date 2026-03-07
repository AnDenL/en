use eframe::egui;

pub fn draw_json_inspector(ui: &mut egui::Ui, name: &str, value: &mut serde_json::Value) -> bool {
    let mut changed = false;

    match value {
        serde_json::Value::Object(map) => {
            ui.collapsing(egui::RichText::new(name).strong(), |ui| {
                for (k, v) in map.iter_mut() {
                    if draw_json_inspector(ui, k, v) {
                        changed = true;
                    }
                }
            });
        }
        serde_json::Value::Array(arr) => {
            if arr.len() == 4 && arr.iter().all(|v| v.is_number()) {
                ui.horizontal(|ui| {
                    ui.label(name);
                    let r = arr[0].as_f64().unwrap_or(1.0) as f32;
                    let g = arr[1].as_f64().unwrap_or(1.0) as f32;
                    let b = arr[2].as_f64().unwrap_or(1.0) as f32;
                    let a = arr[3].as_f64().unwrap_or(1.0) as f32;
                    
                    let mut color = egui::Color32::from_rgba_unmultiplied(
                        (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (a * 255.0) as u8
                    );
                    
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        arr[0] = serde_json::json!(color.r() as f32 / 255.0);
                        arr[1] = serde_json::json!(color.g() as f32 / 255.0);
                        arr[2] = serde_json::json!(color.b() as f32 / 255.0);
                        arr[3] = serde_json::json!(color.a() as f32 / 255.0);
                        changed = true;
                    }
                });
            } else {
                ui.collapsing(name, |ui| {
                    for (i, v) in arr.iter_mut().enumerate() {
                        if draw_json_inspector(ui, &format!("[{}]", i), v) { changed = true; }
                    }
                });
            }
        }
        serde_json::Value::Number(num) => {
            ui.horizontal(|ui| {
                ui.label(name);
                if let Some(val) = num.as_f64() {
                    let mut f = val;
                    if ui.add(egui::DragValue::new(&mut f).speed(0.1)).changed() {
                        if num.is_i64() || num.is_u64() {
                            *num = serde_json::Number::from(f as i64);
                        } else {
                            *num = serde_json::Number::from_f64(f).unwrap();
                        }
                        changed = true;
                    }
                }
            });
        }
        serde_json::Value::String(s) => {
            ui.horizontal(|ui| {
                ui.label(name);
                if ui.text_edit_singleline(s).changed() { changed = true; }
            });
        }
        serde_json::Value::Bool(b) => {
            ui.horizontal(|ui| {
                ui.label(name);
                if ui.checkbox(b, "").changed() { changed = true; }
            });
        }
        serde_json::Value::Null => {
            ui.horizontal(|ui| { ui.label(name); ui.label("null"); });
        }
    }
    changed
}