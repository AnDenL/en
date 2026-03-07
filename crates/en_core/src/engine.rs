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

pub struct EnEngine {
    pub renderer: Renderer,
    pub world: World,
    pub schedule: Schedule,
}

impl EnEngine {
    pub fn new(window: Arc<Window>) -> Self {
        let renderer = pollster::block_on(Renderer::new(window));
        let mut world = World::new();

        world.insert_resource(crate::time::Time::default());
        world.insert_resource(crate::input::Input::default());

        let mut schedule = Schedule::default();

        for sys in inventory::iter::<crate::SystemRegister> {
            (sys.register)(&mut schedule);
            println!("[EnEngine] Auto-registered system: {}", sys.name);
        }

        Self { renderer, world, schedule }
    }

    pub fn init_project(&mut self, project_path: &str) {
        let project_file = std::path::Path::new(project_path).join("en_project.json");
        
        if let Ok(data) = std::fs::read_to_string(&project_file) {
            let json: serde_json::Value = serde_json::from_str(&data).unwrap_or_default();
            let entry_scene = json["entry_scene"].as_str().unwrap_or("main.scene");
            
            let scene_path = std::path::Path::new(project_path).join(entry_scene);
            self.load_scene(scene_path.to_str().unwrap());
        } else {
            eprintln!("[EnEngine] Not found en_project.json у {}", project_path);
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
        if let Some(scene) = Scene::load(path) {
            for entity_data in scene.entities {
                let mut entity = self.world.spawn_empty();
                
                if let Some(val) = entity_data.components.get("Transform") {
                    if let Ok(c) = serde_json::from_value::<Transform>(val.clone()) {
                        entity.insert(c);
                    }
                }
                if let Some(val) = entity_data.components.get("Sprite") {
                    if let Ok(c) = serde_json::from_value::<Sprite>(val.clone()) {
                        entity.insert(c);
                    }
                }
                
                println!("[EnEngine] Spawned entity: {}", entity_data.name);
            }
        }
    }
}

struct EngineApp {
    engine: Option<EnEngine>,
    project_path: String,
}

impl ApplicationHandler for EngineApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.is_none() {
            let window_attr = Window::default_attributes()
                .with_decorations(false)
                .with_transparent(true);
                
            let window = Arc::new(event_loop.create_window(window_attr).unwrap());
            let mut engine = EnEngine::new(window);
            
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

pub fn run(project_path: String) {
    let event_loop = EventLoop::new().unwrap();
    
    event_loop.set_control_flow(ControlFlow::Poll); 
    
    let mut app = EngineApp {
        engine: None,
        project_path,
    };
    
    event_loop.run_app(&mut app).unwrap();
}