use en_core::EnEngine;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct RunnerApp {
    engine: Option<EnEngine>,
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut app = RunnerApp::default();
    
    event_loop.run_app(&mut app).unwrap();
}

impl ApplicationHandler for RunnerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("En Engine - Game Preview")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
            
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            
            self.engine = Some(EnEngine::new(window));
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
                WindowEvent::RedrawRequested => {
                    engine.update();
                    if let Err(fatal_error) = engine.render() {
                        eprintln!("{}", fatal_error);
                        event_loop.exit();
                    }
                }
                _ => {}
            }
        }
    }
    
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(engine) = &self.engine {
            engine.renderer.window.request_redraw();
        }
    }
}