en_core::include_scripts!();
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let project_path = if args.len() > 1 { args[1].clone() } else { ".".to_string() };

    en_core::engine::run(project_path);
}