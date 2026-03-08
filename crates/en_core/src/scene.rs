use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntityData {
    pub name: String,
    pub components: HashMap<String, serde_json::Value>, 
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Scene {
    pub entities: Vec<EntityData>,
}

impl Scene {
    pub fn save(&self, path: &str) {
        let json = serde_json::to_string_pretty(self).expect("Failed to serialize scene");
        fs::write(path, json).expect("Failed to write scene file");
    }

    pub fn load(path: &str) -> Option<Self> {
        match fs::read_to_string(path) {
            Ok(data) => {
                match serde_json::from_str(&data) {
                    Ok(scene) => Some(scene),
                    Err(e) => {
                        eprintln!("[Scene Error] Failed to parse JSON in '{}': {}", path, e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("[Scene Error] Could not read file '{}': {}", path, e);
                None
            }
        }
    }
}