use hecs::World;
use super::ast::{EnCommand, SpawnedByScript};

pub fn run_script(world: &mut World, script_name: &str, bytecode: &[EnCommand]) {
    let mut to_despawn = Vec::new();
    for (entity, marker) in world.query::<&SpawnedByScript>().iter() {
        if marker.script_name == script_name {
            to_despawn.push(entity);
        }
    }
    
    for entity in to_despawn {
        world.despawn(entity).unwrap();
    }
    let mut current_entity = None; 

    for command in bytecode {
        match command {
            EnCommand::SpawnEmpty => {
                let entity = world.spawn((SpawnedByScript {
                    script_name: script_name.to_string(),
                },));
                current_entity = Some(entity);
            }
            EnCommand::AddComponent { name, args } => {
                if let Some(entity) = current_entity {
                    match name.as_str() {
                        "Pos" => {
                            let x = args.get(0).copied().unwrap_or(0.0);
                            let y = args.get(1).copied().unwrap_or(0.0);
                            world.insert_one(entity, crate::components::Pos { x, y }).unwrap();
                        }
                        "Health" => {
                            let hp = args.get(0).copied().unwrap_or(100.0);
                        }
                        _ => println!("[EnS] Попередження: Невідомий компонент '{}'", name),
                    }
                }
            }
        }
    }
}