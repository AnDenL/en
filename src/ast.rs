//use hecs::Entity;

pub struct SpawnedByScript {
    pub script_name: String,
}

#[derive(Debug, Clone)]
pub enum EnCommand {
    SpawnEmpty,
    AddComponent { name: String, args: Vec<f32> },
}