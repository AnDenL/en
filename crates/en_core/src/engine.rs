use bevy_ecs::component::Component;
use bevy_ecs::world::World;
use bevy_ecs::prelude::{Schedule};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use winit::window::Window;
use glam::{Mat4, Quat, Vec3}; 

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, ControlFlow};
use winit::window::WindowId;

use crate::components::{Render, Transform};
use crate::renderer::{Renderer, InstanceRaw};
use crate::time::Time;
use crate::input::Input;
use crate::scene::Scene;
use crate::assets::AssetLoader;

//#[cfg(feature = "editor")]
#[derive(Component)]
pub struct EditorSelected;

pub struct EnEngine {
    pub renderer: Renderer,
    pub world: World,
    pub schedule: Schedule,
    pub inserters: std::collections::HashMap<String, fn(&mut bevy_ecs::world::EntityWorldMut, serde_json::Value)>,
}

impl EnEngine {
    pub async fn new(window: Arc<Window>, plugin_registry: crate::PluginRegistry) -> Self {
        let renderer = Renderer::new(window).await;
        let mut world = World::new();

        world.insert_resource(crate::time::Time::default());
        world.insert_resource(crate::input::Input::default());
        world.insert_resource(crate::texture_manager::SpriteManager::default());

        let mut schedule = Schedule::default();
        let mut inserters = std::collections::HashMap::new();

        for template in inventory::iter::<crate::ComponentTemplate> {
            inserters.insert(template.name.to_string(), template.inserter);
        }

        for template in plugin_registry.components {
            if !inserters.contains_key(template.name) {
                inserters.insert(template.name.to_string(), template.inserter);
                println!("[EnEngine] Registered plugin component: {}", template.name);
            }
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

    pub async fn init_project(&mut self, project_path: &str) {
        let project_dir = std::path::Path::new(project_path);
        
        let project_file = project_dir.join("en_project.json");

        if let Ok(data) = std::fs::read_to_string(&project_file) {
            let json: serde_json::Value = serde_json::from_str(&data).unwrap_or_default();
            let entry_scene = json["entry_scene"].as_str().unwrap_or("main.scene");
            
            self.load_scene(project_path, entry_scene);
        } else {
            eprintln!("[EnEngine] can't load project file: {:?}", project_file);
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
        let instances = self.get_render_instances();
        self.renderer.render(&instances)
    }

    pub fn load_scene(&mut self, project_path: &str, scene_file: &str) {
        let loader_path = format!("{}/", project_path);
        let loader = AssetLoader::new(&loader_path);
        
        let scene_file_clone = scene_file.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(scene) = Scene::load(&loader, &scene_file_clone).await {
                    let _ = tx.send(scene);
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let scene = pollster::block_on(Scene::load(&loader, &scene_file_clone));
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

    // INITIALIZATION FOR THE EDITOR
    // This method will call eframe when the program starts.
    // We are not creating a window here, but taking ready-made video card resources (wgpu),
    // which are kindly provided to us by eframe.
    pub fn new_for_editor(
        device: Arc<wgpu::Device>, 
        queue: Arc<wgpu::Queue>, 
        format: wgpu::TextureFormat,
        plugin_registry: crate::PluginRegistry,
    ) -> Self {
        // Create a renderer specifically for the editor (it has no surface and no window)
        let renderer = Renderer::new_for_editor(device, queue, format);
        let mut world = World::new();

        let mut schedule = Schedule::default();
        let mut inserters = std::collections::HashMap::new();

        let type_registry = bevy_ecs::reflect::AppTypeRegistry::default();
        {
            let mut registry_lock = type_registry.write();
            
            for template in inventory::iter::<crate::ComponentTemplate> {
                inserters.insert(template.name.to_string(), template.inserter);
                
                (template.register_type)(&mut registry_lock);
            }

            for template in plugin_registry.components {
                if !inserters.contains_key(template.name) {
                    inserters.insert(template.name.to_string(), template.inserter);
                    
                    (template.register_type)(&mut registry_lock);
                }
            }
        }

        world.insert_resource(type_registry);
        world.insert_resource(crate::time::Time::default());
        world.insert_resource(crate::input::Input::default());
        world.insert_resource(crate::texture_manager::SpriteManager::default());

        for sys in inventory::iter::<crate::SystemRegister> {
            (sys.register)(&mut schedule);
        }

        for sys in plugin_registry.systems {
            (sys.register)(&mut schedule);
        }

        Self { renderer, world, schedule, inserters }
    }

    // This method collects all data for wgpu directly from ECS memory.
    pub fn get_render_instances(&mut self) -> Vec<InstanceRaw> {
        let mut instances = Vec::new();
        
        // Querying ECS for Transform and Render components.
        // We use Option<&EditorSelected> so the query includes entities even without the selection marker.
        let mut query = self.world.query::<(
            &Transform, 
            &Render, 
            Option<&EditorSelected>
        )>();
        
        for (transform, render, selected) in query.iter(&self.world) {
            // Create base transformation matrix using glam.
            // Assuming your Transform uses 2D coordinates (x, y) and rotation in radians.
            let position = Vec3::new(transform.x, transform.y, 0.0);
            let rotation = Quat::from_rotation_z(transform.rotation);
            let scale = Vec3::ONE; // Default scale, can be replaced with transform.scale if available

            let model_matrix = Mat4::from_scale_rotation_translation(scale, rotation, position);

            // Convert glam Mat4 to the raw nested array [[f32; 4]; 4] for wgpu.
            let model = model_matrix.to_cols_array_2d();
            let color_arr = render.color.to_array();

            // --- Editor Selection Outline Logic ---
            if selected.is_some() {
                let outline_scale_val = 1.05;
                
                // Create a slightly larger matrix for the outline.
                let outline_matrix = Mat4::from_scale_rotation_translation(
                    Vec3::splat(outline_scale_val), 
                    rotation, 
                    position
                );
                
                instances.push(InstanceRaw { 
                    model: outline_matrix.to_cols_array_2d(), 
                    color: [1.0, 0.8, 0.0, 1.0] // Yellow highlight
                });
            }

            // Add the actual sprite instance.
            // If the outline was added, this will be rendered on top (depending on your pipeline/depth).
            instances.push(InstanceRaw { 
                model, 
                color: color_arr 
            });
        }
        
        instances
    }
}

struct EngineApp {
    engine: Rc<RefCell<Option<EnEngine>>>,
    project_path: String,
    plugin_registry: Option<crate::PluginRegistry>,
}

impl ApplicationHandler for EngineApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let engine_is_none = self.engine.borrow().is_none();
        
        if engine_is_none {
            let window_attr = Window::default_attributes()
                .with_decorations(false)
                .with_transparent(true);
                
            let window = Arc::new(event_loop.create_window(window_attr).unwrap());
            
            let registry = self.plugin_registry.take().unwrap_or(crate::PluginRegistry {
                components: vec![],
                systems: vec![],
            });
            let project_path = self.project_path.clone();
            #[cfg(target_arch = "wasm32")]
            let engine_handle = self.engine.clone();

            #[cfg(not(target_arch = "wasm32"))]
            {
                let mut engine = pollster::block_on(EnEngine::new(window, registry));
                pollster::block_on(engine.init_project(&project_path));
                *self.engine.borrow_mut() = Some(engine);
            }

            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen_futures::spawn_local;
                
                spawn_local(async move {
                    let mut engine = EnEngine::new(window, registry).await;
                    engine.init_project(&project_path).await;
                    *engine_handle.borrow_mut() = Some(engine);
                });
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
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
        if let Some(engine) = &mut *self.engine.borrow_mut() {
            engine.update();
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
        engine: Rc::new(RefCell::new(None)),
        project_path,
        plugin_registry: Some(plugin_registry),
    };
    
    event_loop.run_app(&mut app).unwrap();
}