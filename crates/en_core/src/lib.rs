pub mod components;
pub mod engine;
pub mod renderer;
pub mod camera;
pub mod time;
pub mod input;
pub mod scene;

pub use engine::EnEngine;
pub use en_macros::en_system;
pub use inventory;
pub use bevy_ecs;
pub use en_macros::include_scripts;

pub struct SystemRegister {
    pub name: &'static str,
    pub register: fn(&mut bevy_ecs::schedule::Schedule),
}

inventory::collect!(SystemRegister);

pub mod prelude {
    pub use bevy_ecs::prelude::*;
    pub use crate::components::*;
    pub use crate::input::*;
    pub use crate::time::*;
    pub use crate::en_system;
}