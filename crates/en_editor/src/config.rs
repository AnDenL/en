use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct EditorConfig {
    pub(crate) last_project_path: Option<String>,
}

/// Finds the correct folder to store our editor config (e.g., AppData on Windows, ~/.config on Linux).
pub fn get_editor_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "en", "EnEngine") {
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).unwrap(); // Ensure the directory exists
        config_dir.join("editor_config.json")
    } else {
        panic!("Could not find system folder for configs!");
    }
}

pub fn load_editor_config() -> EditorConfig {
    let path = get_editor_config_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str(&data) {
                return config;
            }
        }
    }
    EditorConfig::default()
}

pub fn save_editor_config(config: &EditorConfig) {
    let path = get_editor_config_path();
    if let Ok(data) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, data);
    }
}
