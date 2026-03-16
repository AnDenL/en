use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Resource)]
pub struct ProjectConfig {
    #[serde(default = "default_entry_scene")]
    pub entry_scene: String,
    
    #[serde(default)]
    pub render: RenderConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            entry_scene: default_entry_scene(),
            render: RenderConfig::default(),
        }
    }
}

fn default_entry_scene() -> String { "main.scene".to_string() }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RenderConfig {
    pub clear_color: [f32; 4],
    pub vsync: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            clear_color: [0.1, 0.1, 0.15, 1.0],
            vsync: true,
        }
    }
}