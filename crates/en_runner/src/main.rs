en_core::include_scripts!();

fn main() {
    let mut components = Vec::new();
    let mut systems = Vec::new();

    for template in en_core::inventory::iter::<en_core::ComponentTemplate> {
        components.push(template.clone());
    }
    
    for sys in en_core::inventory::iter::<en_core::SystemRegister> {
        systems.push(sys.clone());
    }

    let registry = en_core::PluginRegistry {
        components,
        systems,
    };
    
    let project_path = std::env::var("PROJECT_DIR").unwrap_or_else(|_| ".".to_string());
    en_core::engine::run(project_path, registry);
}