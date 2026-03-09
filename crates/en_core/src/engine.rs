use bevy_ecs::world::World;
use bevy_ecs::prelude::{Schedule};
use std::sync::Arc;
use winit::window::Window;
use glam::{Mat4, Quat, Vec3}; 

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, ControlFlow};
use winit::window::WindowId;

use crate::components::{Sprite, Transform};
use crate::renderer::{Renderer, InstanceRaw};
use crate::time::Time;
use crate::input::Input;
use crate::scene::Scene;
use crate::assets::AssetLoader;

pub struct EnEngine {
    pub renderer: Renderer,
    pub world: World,
    pub schedule: Schedule,
    pub inserters: std::collections::HashMap<String, fn(&mut bevy_ecs::world::EntityWorldMut, serde_json::Value)>,
}

impl EnEngine {
    pub fn new(window: Arc<Window>, plugin_registry: crate::PluginRegistry) -> Self {
        let renderer = pollster::block_on(Renderer::new(window));
        let mut world = World::new();

        world.insert_resource(crate::time::Time::default());
        world.insert_resource(crate::input::Input::default());

        let mut schedule = Schedule::default();
        let mut inserters = std::collections::HashMap::new();

        for template in inventory::iter::<crate::ComponentTemplate> {
            inserters.insert(template.name.to_string(), template.inserter);
        }

        for template in plugin_registry.components {
            inserters.insert(template.name.to_string(), template.inserter);
            println!("[EnEngine] Registered plugin component: {}", template.name);
        }

        for sys in inventory::iter::<crate::SystemRegister> {
            (sys.register)(&mut schedule);
            println!("[EnEngine] Auto-registered core system: {}", sys.name);
        }

        for sys in plugin_registry.systems {
            (sys.register)(&mut schedule);
            println!("[EnEngine] Registered plugin system: {}", sys.name);
        }

        Self { renderer, world, schedule, inserters }
    }

    pub fn init_project(&mut self, project_path: &str) {
        let project_file = std::path::Path::new(project_path).join("en_project.json");
        
        if let Ok(data) = std::fs::read_to_string(&project_file) {
            let json: serde_json::Value = serde_json::from_str(&data).unwrap_or_default();
            let entry_scene = json["entry_scene"].as_str().unwrap_or("main.scene");
            
            let scene_path = std::path::Path::new(project_path).join(entry_scene);
            self.load_scene(scene_path.to_str().unwrap());
        } else {
            eprintln!("[EnEngine] Not found en_project.json {}", project_path);
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    pub fn update(&mut self) {
        if let Some(mut time) = self.world.get_resource_mut::<Time>() {
            time.update();
        }

        self.schedule.run(&mut self.world);

        if let Some(mut input) = self.world.get_resource_mut::<Input>() {
            input.clear_frame();
        }
    }

    pub fn render(&mut self) -> Result<(), &'static str> {
        let mut instances = Vec::new();
        let mut query = self.world.query::<(&Transform, &Sprite)>();
        
        for (transform, sprite) in query.iter(&self.world) {
            let model_matrix = Mat4::from_scale_rotation_translation(
                Vec3::ONE, 
                Quat::from_rotation_z(transform.rotation), 
                Vec3::new(transform.x, transform.y, 0.0)
            );

            instances.push(InstanceRaw {
                model: model_matrix.to_cols_array_2d(),
                color: sprite.color,
            });
        }

        self.renderer.render(&instances)
    }

    pub fn load_scene(&mut self, path: &str) {
        let path_clone = path.to_string();
        let loader = AssetLoader::new("assets/");
        
        let (tx, rx) = std::sync::mpsc::channel();

        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(scene) = Scene::load(&loader, &path_clone).await {
                    let _ = tx.send(scene);
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let scene = pollster::block_on(Scene::load(&loader, &path_clone));
            if let Some(s) = scene { let _ = tx.send(s); }
        }

        if let Ok(scene) = rx.try_recv() {
            self.apply_scene(Some(scene));
        }
    }

    fn apply_scene(&mut self, scene: Option<Scene>) {
    if let Some(scene) = scene {
        for entity_data in scene.entities {
            let mut entity = self.world.spawn_empty();
            
            for (comp_name, comp_value) in entity_data.components {
                if let Some(inserter) = self.inserters.get(comp_name.as_str()) {
                    (inserter)(&mut entity, comp_value);
                } else {
                    eprintln!("[EnEngine Warning] Unknown component: {}", comp_name);
                }
            }
        }
    }
}
}

struct EngineApp {
    engine: Option<EnEngine>,
    project_path: String,
    plugin_registry: Option<crate::PluginRegistry>,
}

impl ApplicationHandler for EngineApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.is_none() {
            let window_attr = Window::default_attributes()
                .with_decorations(false)
                .with_transparent(true);
                
            let window = Arc::new(event_loop.create_window(window_attr).unwrap());
            
            let registry = self.plugin_registry.take().unwrap_or(crate::PluginRegistry {
                components: vec![],
                systems: vec![],
            });

            let mut engine = EnEngine::new(window, registry);
            
            engine.init_project(&self.project_path);
            self.engine = Some(engine);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(engine) = &mut self.engine {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    engine.resize(physical_size);
                }
                WindowEvent::KeyboardInput { event: winit::event::KeyEvent { physical_key: winit::keyboard::PhysicalKey::Code(keycode), state, .. }, .. } => {
                    if let Some(mut input) = engine.world.get_resource_mut::<crate::input::Input>() {
                        if state == winit::event::ElementState::Pressed {
                            input.press(keycode);
                        } else {
                            input.release(keycode);
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    engine.update();
                    if let Err(fatal_error) = engine.render() {
                        eprintln!("{}", fatal_error);
                        event_loop.exit();
                    }
                    
                    if let Some(window) = &engine.renderer.window {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        }
    }
    
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(engine) = &self.engine {
            if let Some(window) = &engine.renderer.window {
                window.request_redraw();
            }
        }
    }
}

pub fn run(project_path: String, plugin_registry: crate::PluginRegistry) {
    let event_loop = EventLoop::new().unwrap();
    
    event_loop.set_control_flow(ControlFlow::Poll); 
    
    let mut app = EngineApp {
        engine: None,
        project_path,
        plugin_registry: Some(plugin_registry),
    };
    
    event_loop.run_app(&mut app).unwrap();
}