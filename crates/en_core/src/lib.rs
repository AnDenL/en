pub mod components;
pub mod engine;
pub mod renderer;
pub mod camera;
pub mod time;
pub mod input;
pub mod scene;
pub mod assets;
pub mod types;
pub mod texture_manager;

pub use engine::EnEngine;
pub use en_macros::*;
pub use inventory;
pub use bevy_ecs;
pub use smart_default;
pub use serde_json;

pub use types::*;
pub use texture_manager::{SpriteData, SpriteId, SpriteManager};

#[doc(hidden)]
extern crate self as en_core;

#[derive(Clone)]
pub struct ComponentTemplate {
    pub name: &'static str,
    pub generator: fn() -> serde_json::Value,
    pub inserter: fn(&mut bevy_ecs::world::EntityWorldMut, serde_json::Value),
    pub schema: fn() -> serde_json::Value,
}

#[derive(Clone)]
pub struct SystemRegister {
    pub name: &'static str,
    pub register: fn(&mut bevy_ecs::schedule::Schedule),
}
pub struct PluginRegistry {
    pub components: Vec<ComponentTemplate>,
    pub systems: Vec<SystemRegister>,
}

inventory::collect!(ComponentTemplate);
inventory::collect!(SystemRegister);

pub mod prelude {
    pub use bevy_ecs::prelude::*;
    pub use crate::bevy_ecs;
    pub use crate::components::*;
    pub use crate::input::*;
    pub use crate::time::*;
    pub use crate::inventory;
    pub use crate::smart_default;
    pub use crate::serde_json;
    pub use crate::en_system;
    pub use crate::en_component;


    pub use crate::types::*;
}