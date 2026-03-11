use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use en_core::bevy_ecs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use en_core::engine::EnEngine;
use crate::panels;

// ----------------------------------------------------------------------------
// DOCKING SYSTEM DEFINITIONS
// ----------------------------------------------------------------------------

/// Represents all possible dockable tabs in our editor.
#[derive(Debug, PartialEq, Clone, Default)]
pub enum EditorTab {
    #[default]
    Viewport,
    SceneTree,
    Inspector,
    Assets,
    Console,
}

// We need access to `eframe::Frame` inside our tabs to register wgpu textures.
struct EditorTabViewer<'a, 'b> {
    app: &'a mut EditorApp,
    frame: &'b mut eframe::Frame,
}

impl<'a, 'b> TabViewer for EditorTabViewer<'a, 'b> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::Viewport => "🎮 Viewport".into(),
            EditorTab::SceneTree => "🌲 Scene Tree".into(),
            EditorTab::Inspector => "⚙ Inspector".into(),
            EditorTab::Assets => "📁 Assets".into(),
            EditorTab::Console => "💻 Console".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::Assets => panels::assets::draw(ui, self.app),
            EditorTab::Console => panels::console::draw(ui, self.app),
            EditorTab::Viewport => panels::viewport::draw(ui, self.app, self.frame),
            EditorTab::SceneTree => panels::scene_tree::draw(ui, self.app),
            
            EditorTab::Inspector => panels::inspector::draw(ui, self.app),
            
        }
    }
}

// ----------------------------------------------------------------------------
// EDITOR STATE
// ----------------------------------------------------------------------------

pub struct EditorUiState {
    pub project_path: String,
    pub is_playing: bool,
    pub selected_entity: Option<bevy_ecs::entity::Entity>, 

    // The state of our dock layout (which tab is where)
    pub dock_state: DockState<EditorTab>,

    pub current_asset_path: PathBuf,
    pub last_asset_path: PathBuf,
    pub asset_cache: Vec<(PathBuf, String, bool)>, 
    pub logs: Arc<Mutex<Vec<String>>>,
    
    // We can add build process state here later

    pub viewport_texture: Option<wgpu::Texture>,
    pub viewport_texture_id: Option<egui::TextureId>,
}

pub struct EditorApp {
    pub engine: EnEngine,
    pub ui_state: EditorUiState,
}

impl EditorApp {
    pub fn new(
        project_path: String, 
        device: Arc<wgpu::Device>, 
        queue: Arc<wgpu::Queue>, 
        target_format: wgpu::TextureFormat
    ) -> Self {
        
        let game_registry = en_core::PluginRegistry { components: vec![], systems: vec![] };
        
        // TODO: Load DLL logic here...

        let mut engine = EnEngine::new_for_editor(device, queue, target_format, game_registry);
        pollster::block_on(engine.init_project(&project_path));

        {
            let registry = engine.world.resource::<bevy_ecs::reflect::AppTypeRegistry>();
            let mut lock = registry.write();
            macro_rules! register_primitive {
                ($($t:ty),+) => {
                    $(
                        lock.register::<$t>();
                        lock.register_type_data::<$t, bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl>();
                    )+
                };
            }
            register_primitive!(
                f32, f64,
                i8, i16, i32, i64, isize,
                u8, u16, u32, u64, usize,
                bool, String
            );
        }

        // Setup default Dock Layout (Unity/Godot style)
        // 1. Center = Viewport
        let mut dock_state = DockState::new(vec![EditorTab::Viewport]);
        let surface = dock_state.main_surface_mut();
        
        // 2. Right = Inspector
        let [main, _right] = surface.split_right(NodeIndex::root(), 0.75, vec![EditorTab::Inspector]);
        
        // 3. Left = Scene Tree
        let [_left, center] = surface.split_left(main, 0.2, vec![EditorTab::SceneTree]);
        
        // 4. Bottom = Assets and Console (as tabs in the same window!)
        let [_viewport, _bottom] = surface.split_below(center, 0.7, vec![EditorTab::Assets, EditorTab::Console]);

        let ui_state = EditorUiState {
            project_path: project_path.clone(),
            selected_entity: None,
            is_playing: false,
            dock_state,
            current_asset_path: PathBuf::from(&project_path),
            last_asset_path: PathBuf::from("."),
            asset_cache: Vec::new(),
            logs: Arc::new(Mutex::new(Vec::new())),
            viewport_texture: None,
            viewport_texture_id: None,
        };

        Self { engine, ui_state }
    }

    // --- Helper Methods ---
    pub fn refresh_asset_cache(&mut self) {
        self.ui_state.asset_cache.clear();
        if let Ok(entries) = std::fs::read_dir(&self.ui_state.current_asset_path) {
            let mut entries_vec: Vec<_> = entries.flatten().collect();
            entries_vec.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));

            for entry in entries_vec {
                let path = entry.path();
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let is_dir = path.is_dir();
                self.ui_state.asset_cache.push((path, file_name, is_dir));
            }
        }
        self.ui_state.last_asset_path = self.ui_state.current_asset_path.clone();
    }
    
    pub fn log(&self, message: &str) {
        if let Ok(mut logs) = self.ui_state.logs.lock() {
            logs.push(format!("[Editor] {}", message));
        }
    }
}

// ----------------------------------------------------------------------------
// EFRAME UPDATE LOOP
// ----------------------------------------------------------------------------
impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        egui::TopBottomPanel::top("top_bar")
            .frame(en_ui::theme::bar_frame())
            .show(ctx, |ui| {
                panels::top_bar::draw(ctx, ui, self);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE) 
            .show(ctx, |ui| {
                let placeholder = DockState::new(vec![]); 

                let mut current_dock_state = std::mem::replace(&mut self.ui_state.dock_state, placeholder);
                
                {
                    let mut viewer = EditorTabViewer { app: self, frame: _frame };
                    
                    DockArea::new(&mut current_dock_state)
                        .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
                        .show_inside(ui, &mut viewer);
                }

                self.ui_state.dock_state = current_dock_state;
            });

        if self.ui_state.is_playing {
            self.engine.update();
            ctx.request_repaint();
        }
    }
}