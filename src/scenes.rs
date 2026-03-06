use hecs::World;
use macroquad::prelude::*;

use crate::compiler::compile_source;
use crate::runner::run_script;
use crate::components::load_scene;

pub struct SceneManager;

impl SceneManager {
    pub async fn load_level(world: &mut World, level_name: &str) {
        let bin_path = format!("{}.bin", level_name);
        let ens_path = format!("{}.ens", level_name);
        if let Ok(bytes) = load_file(&bin_path).await {
            load_scene(world, &bytes);
            
            for (_id, ren) in world.query_mut::<&mut crate::components::Render>() {
                ren.cached_sprite = None;
            }
            println!("[SceneManager] Binary scene {} loaded!", bin_path);
        } else {
            println!("[SceneManager] Warning: {} not found.", bin_path);
        }

        if let Ok(bytes) = load_file(&ens_path).await {
            if let Ok(source_code) = String::from_utf8(bytes) {
                println!("[SceneManager] Found script {}, compiling...", ens_path);
                
                let bytecode = compile_source(&source_code, "");
                
                run_script(world, level_name, &bytecode);
                
                println!("[SceneManager] Script {} executed!", ens_path);
            }
        } else {
            println!("[SceneManager] Script {} not found, skip.", ens_path);
        }
    }
}

#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
pub struct ScriptWatcher {
    pub script_name: String,
    last_modified: std::time::SystemTime,
}

#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
impl ScriptWatcher {
    pub fn new(script_name: &str) -> Self {
        let path = format!("{}.ens", script_name);
        
        let last_modified = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

        Self {
            script_name: script_name.to_string(),
            last_modified,
        }
    }

    pub async fn update(&mut self, world: &mut World) {
        let path = format!("{}.ens", self.script_name);
        
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                if modified > self.last_modified {
                    println!("[Hot-Reload] 🔄 File {} changed! Reload script...", path);
                    self.last_modified = modified;

                    if let Ok(bytes) = load_file(&path).await {
                        if let Ok(source_code) = String::from_utf8(bytes) {
                            
                            let bytecode = crate::compiler::compile_source(&source_code, "assets/scripts");
                            crate::runner::run_script(world, &self.script_name, &bytecode);
                            
                            println!("[Hot-Reload] ✅ Successfully updated!");
                        }
                    }
                }
            }
        }
    }
}