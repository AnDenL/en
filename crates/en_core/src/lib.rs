pub mod assets;
pub mod components;
pub mod config;
pub mod engine;
pub mod input;
pub mod scene;
pub mod time;
pub mod types;

pub use bevy_ecs;
pub use bevy_reflect;
pub use en_macros::*;
pub use engine::EnEngine;
pub use inventory;
pub use serde_json;
pub use smart_default;

pub use types::*;

#[doc(hidden)]
extern crate self as en_core;

#[derive(Clone)]
pub struct ComponentTemplate {
    pub name: &'static str,
    pub generator: fn() -> serde_json::Value,
    pub inserter: fn(&mut bevy_ecs::world::EntityWorldMut, serde_json::Value),

    pub register_type: fn(&mut bevy_reflect::TypeRegistry),
}

#[derive(Clone)]
pub struct SystemRegister {
    pub name: &'static str,
    pub register: fn(&mut bevy_ecs::schedule::Schedule),
}

#[derive(Default)]
pub struct PluginRegistry {
    pub components: Vec<ComponentTemplate>,
    pub systems: Vec<SystemRegister>,
}

inventory::collect!(ComponentTemplate);
inventory::collect!(SystemRegister);

pub mod prelude {
    pub use bevy_ecs::prelude::*;
    pub use bevy_reflect::prelude::*;

    pub use crate::bevy_ecs;
    pub use crate::bevy_reflect;
    pub use crate::components::Name;
    pub use crate::components::*;
    pub use crate::en_component;
    pub use crate::en_system;
    pub use crate::input::*;
    pub use crate::inventory;
    pub use crate::serde_json;
    pub use crate::smart_default;
    pub use crate::time::*;
    pub use crate::types::*;
}
