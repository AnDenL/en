use bevy_ecs::component::Component;
use bevy_ecs::reflect::AppTypeRegistry;
use bevy_ecs::world::World;
use bevy_ecs::prelude::Schedule;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use winit::window::Window;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, ControlFlow};
use winit::window::WindowId;

use crate::config::ProjectConfig;
use crate::renderer::{Renderer, build_render_batches};
use crate::time::Time;
use crate::input::Input;
use crate::scene::Scene;
use crate::assets::AssetLoader;

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
        world.insert_resource(crate::texture_manager::TextureManager::default());
        world.insert_resource(ProjectConfig::default());

        let mut schedule = Schedule::default();
        let mut inserters = std::collections::HashMap::new();

        for template in inventory::iter::<crate::ComponentTemplate> {
            inserters.insert(template.name.to_string(), template.inserter);
        }

        for template in plugin_registry.components {
            if !inserters.contains_key(template.name) {
                inserters.insert(template.name.to_string(), template.inserter);
            }
        }

        for sys in inventory::iter::<crate::SystemRegister> {
            (sys.register)(&mut schedule);
        }

        for sys in plugin_registry.systems {
            (sys.register)(&mut schedule);
        }

        Self { renderer, world, schedule, inserters }
    }

    pub async fn init_project(&mut self, project_path: &str) {
        let loader_path = format!("{}/", project_path);
        let loader = AssetLoader::new(&loader_path);
        
        let config_result = loader.load_json::<ProjectConfig>("en_project.json").await;
        
        match config_result {
            Ok(config) => {
                let entry_scene = config.entry_scene.clone();
                self.world.insert_resource(config);
                self.load_scene(project_path, &entry_scene);
            }
            Err(_) => {
                self.load_scene(project_path, "main.scene");
            }
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
        let batches = build_render_batches(&mut self.world, &self.renderer);
        self.renderer.render(&batches)
    }

    pub fn render_editor_view(&mut self, view: &wgpu::TextureView) {
        let batches = build_render_batches(&mut self.world, &self.renderer);
        self.renderer.render_to_view(&batches, view)
    }

    pub fn load_scene(&mut self, project_path: &str, scene_file: &str) {
        let loader_path = format!("{}/", project_path);
        let loader = AssetLoader::new(&loader_path);
        let scene_file_clone = scene_file.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(scene) = Scene::load(&loader, &scene_file_clone).await { let _ = tx.send(scene); }
        });

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
                    }
                }
            }
        }
    }

    pub fn new_for_editor(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, format: wgpu::TextureFormat, plugin_registry: crate::PluginRegistry) -> Self {
        let renderer = Renderer::new_for_editor(device, queue, format);
        let mut world = World::new();

        let mut schedule = Schedule::default();
        let mut inserters = std::collections::HashMap::new();

        let type_registry = AppTypeRegistry::default();
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
        world.insert_resource(crate::texture_manager::TextureManager::default());
        world.insert_resource(ProjectConfig::default()); 

        for sys in inventory::iter::<crate::SystemRegister> { (sys.register)(&mut schedule); }
        for sys in plugin_registry.systems { (sys.register)(&mut schedule); }

        Self { renderer, world, schedule, inserters }
    }

    pub fn reload_plugins(&mut self, plugin_registry: crate::PluginRegistry) {
        let mut new_schedule = bevy_ecs::prelude::Schedule::default();

        for sys in inventory::iter::<crate::SystemRegister> { (sys.register)(&mut new_schedule); }
        for sys in plugin_registry.systems { (sys.register)(&mut new_schedule); }

        self.schedule = new_schedule;

        let type_registry = self.world.resource::<AppTypeRegistry>();
        let mut registry_lock = type_registry.write();

        for template in plugin_registry.components {
            self.inserters.insert(template.name.to_string(), template.inserter);
            (template.register_type)(&mut registry_lock);
        }
    }
}

struct EngineApp {
    engine: Rc<RefCell<Option<EnEngine>>>,
    project_path: String,
    plugin_registry: Option<crate::PluginRegistry>,
}

impl ApplicationHandler for EngineApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.borrow().is_none() {
            let window_attr = Window::default_attributes().with_decorations(false).with_transparent(true);
            let window = Arc::new(event_loop.create_window(window_attr).unwrap());
            let registry = self.plugin_registry.take().unwrap_or(crate::PluginRegistry { components: vec![], systems: vec![] });
            let project_path = self.project_path.clone();

            #[cfg(not(target_arch = "wasm32"))]
            {
                let mut engine = pollster::block_on(EnEngine::new(window, registry));
                pollster::block_on(engine.init_project(&project_path));
                *self.engine.borrow_mut() = Some(engine);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(physical_size) => engine.resize(physical_size),
                WindowEvent::KeyboardInput { event: winit::event::KeyEvent { physical_key: winit::keyboard::PhysicalKey::Code(keycode), state, .. }, .. } => {
                    if let Some(mut input) = engine.world.get_resource_mut::<crate::input::Input>() {
                        if state == winit::event::ElementState::Pressed { input.press(keycode); } else { input.release(keycode); }
                    }
                }
                WindowEvent::RedrawRequested => {
                    engine.update();
                    if let Err(_) = engine.render() { event_loop.exit(); }
                    if let Some(window) = &engine.renderer.window { window.request_redraw(); }
                }
                _ => {}
            }
        }
    }
    
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(engine) = &mut *self.engine.borrow_mut() {
            engine.update();
            if let Some(window) = &engine.renderer.window { window.request_redraw(); }
        }
    }
}

pub fn run(project_path: String, plugin_registry: crate::PluginRegistry) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll); 
    let mut app = EngineApp { engine: Rc::new(RefCell::new(None)), project_path, plugin_registry: Some(plugin_registry) };
    event_loop.run_app(&mut app).unwrap();
}