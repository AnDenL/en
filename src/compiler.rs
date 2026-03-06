use macroquad::prelude::load_file;
use crate::ast::EnCommand;

pub fn compile_source(source: &str, _base_path: &str) -> Vec<EnCommand> {
    let mut bytecode = Vec::new();

    for word in source.split_whitespace() {
        if word.starts_with("entity") {
            bytecode.push(EnCommand::SpawnEmpty);

            if word.contains(':') {
                let parts: Vec<&str> = word.split(':').collect();
                
                for comp_str in &parts[1..] {
                    if let Some(bracket_idx) = comp_str.find('(') {
                        let name = comp_str[..bracket_idx].to_string();
                        let args_str = &comp_str[bracket_idx + 1 .. comp_str.len() - 1];
                        
                        let args: Vec<f32> = args_str.split(',')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();
                            
                        bytecode.push(EnCommand::AddComponent { name, args });
                    } else {
                        bytecode.push(EnCommand::AddComponent { 
                            name: comp_str.to_string(), 
                            args: vec![] 
                        });
                    }
                }
            }
        }
    }
    bytecode
}