use eframe::{egui, wgpu};
use egui_tiles::{Behavior, Linear, LinearDir, TileId, Tree, UiResponse};
use en_core::bevy_ecs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::panels::{self, inspector};
use en_core::engine::EnEngine;

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

struct EditorBehavior<'a, 'b> {
    app: &'a mut EditorApp,
    frame: &'b mut eframe::Frame,
}

impl<'a, 'b> Behavior<EditorTab> for EditorBehavior<'a, 'b> {
    fn pane_ui(&mut self, ui: &mut egui::Ui, _tile_id: TileId, tab: &mut EditorTab) -> UiResponse {
        match tab {
            EditorTab::Assets => panels::assets::draw(ui, self.app),
            EditorTab::Console => panels::console::draw(ui, self.app),
            EditorTab::Viewport => panels::viewport::draw(ui, self.app, self.frame),
            EditorTab::SceneTree => panels::scene_tree::draw(ui, self.app),
            EditorTab::Inspector => panels::inspector::draw(ui, self.app),
        }

        // Вказуємо, що ми не виконуємо спеціальних дій (напр. закриття)
        UiResponse::None
    }

    fn tab_title_for_pane(&mut self, tab: &EditorTab) -> egui::WidgetText {
        match tab {
            EditorTab::Viewport => "🎮 Viewport".into(),
            EditorTab::SceneTree => "🌲 Scene Tree".into(),
            EditorTab::Inspector => "⚙ Inspector".into(),
            EditorTab::Assets => "📁 Assets".into(),
            EditorTab::Console => "💻 Console".into(),
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

    pub is_building: Arc<AtomicBool>,
    pub build_receiver: Option<Receiver<bool>>,
    pub active_plugin_lib: Option<libloading::Library>,

    pub tree: Tree<EditorTab>,

    pub current_asset_path: PathBuf,
    pub last_asset_path: PathBuf,
    pub asset_cache: Vec<(PathBuf, String, bool)>,
    pub logs: Arc<Mutex<Vec<String>>>,

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
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let game_registry = en_core::PluginRegistry {
            components: vec![],
            systems: vec![],
        };

        let mut engine = EnEngine::new_headless(device, queue, target_format, game_registry);
        pollster::block_on(engine.init_project(&project_path));

        inspector::setup_inspector_registry(&mut engine.world);

        // --- ІНІЦІАЛІЗАЦІЯ ДЕРЕВА EGUI_TILES ---
        let mut tree = Tree::empty("editor_tree");

        // 1. Створюємо тайли для кожної панелі
        let viewport = tree.tiles.insert_pane(EditorTab::Viewport);
        let scene_tree = tree.tiles.insert_pane(EditorTab::SceneTree);
        let inspector = tree.tiles.insert_pane(EditorTab::Inspector);
        let assets = tree.tiles.insert_pane(EditorTab::Assets);
        let console = tree.tiles.insert_pane(EditorTab::Console);

        // 2. Групуємо вкладки "Ассети" та "Консоль" знизу
        let bottom_tabs = tree.tiles.insert_tab_tile(vec![assets, console]);

        // 3. Формуємо центральну колонку (В'юпорт зверху, вкладки знизу)
        let mut center_linear = Linear::new(LinearDir::Vertical, vec![viewport, bottom_tabs]);
        center_linear.shares.set_share(viewport, 0.75);
        center_linear.shares.set_share(bottom_tabs, 0.25);
        let center_col = tree.tiles.insert_container(center_linear);

        // 4. Формуємо головний горизонтальний макет (Дерево зліва, Центр посередині, Інспектор справа)
        let mut root_linear = Linear::new(
            LinearDir::Horizontal,
            vec![scene_tree, center_col, inspector],
        );
        root_linear.shares.set_share(scene_tree, 0.2);
        root_linear.shares.set_share(center_col, 0.6);
        root_linear.shares.set_share(inspector, 0.2);
        let root_id = tree.tiles.insert_container(root_linear);

        // Встановлюємо корінь дерева
        tree.root = Some(root_id);

        let ui_state = EditorUiState {
            project_path: project_path.clone(),
            selected_entity: None,
            is_playing: false,
            is_building: Arc::new(AtomicBool::new(false)),
            build_receiver: None,
            active_plugin_lib: None,
            tree,
            current_asset_path: PathBuf::from(&project_path),
            last_asset_path: PathBuf::from("."),
            asset_cache: Vec::new(),
            logs: Arc::new(Mutex::new(Vec::new())),
            viewport_texture: None,
            viewport_texture_id: None,
        };

        let mut app = Self { engine, ui_state };
        app.reload_dll();
        app
    }

    pub fn refresh_asset_cache(&mut self) {
        self.ui_state.asset_cache.clear();
        if let Ok(entries) = std::fs::read_dir(&self.ui_state.current_asset_path) {
            let mut entries_vec: Vec<_> = entries.flatten().collect();
            entries_vec.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));

            for entry in entries_vec {
                let path = entry.path();
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
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

    pub fn trigger_build(&mut self, ctx: &egui::Context) {
        if self.ui_state.is_building.load(Ordering::SeqCst) {
            self.log("⚠ Compilation is already running!");
            return;
        }

        self.ui_state.is_building.store(true, Ordering::SeqCst);
        self.log("🔨 Running cargo build...");

        let (tx, rx) = channel();
        self.ui_state.build_receiver = Some(rx);

        let project_path = self.ui_state.project_path.clone();
        let is_building_clone = self.ui_state.is_building.clone();
        let ctx_clone = ctx.clone();

        std::thread::spawn(move || {
            let status = std::process::Command::new("cargo")
                .arg("build")
                .arg("--lib")
                .current_dir(project_path)
                .status();

            let success = match status {
                Ok(s) => s.success(),
                _ => false,
            };

            let _ = tx.send(success);
            is_building_clone.store(false, Ordering::SeqCst);
            ctx_clone.request_repaint();
        });
    }

    pub fn check_build_status(&mut self) {
        if let Some(rx) = &self.ui_state.build_receiver {
            if let Ok(success) = rx.try_recv() {
                self.ui_state.build_receiver = None;
                if success {
                    self.log("✅ Compilation successful! Performing Domain Reload...");
                    self.reload_dll();
                } else {
                    self.log("❌ Compilation error! Check the console.");
                }
            }
        }
    }

    fn reload_dll(&mut self) {
        let project_dir = std::path::Path::new(&self.ui_state.project_path);
        let json_path = project_dir.join("en_project.json");

        let mut proj_name = String::from("game");
        if let Ok(data) = std::fs::read_to_string(&json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(n) = json["project_name"].as_str() {
                    proj_name = n.to_lowercase().replace("-", "_").replace(" ", "_");
                }
            }
        }

        let prefix = std::env::consts::DLL_PREFIX;
        let ext = std::env::consts::DLL_EXTENSION;
        let original_lib_name = format!("{}{}.{}", prefix, proj_name, ext);
        let target_debug_dir = project_dir.join("target").join("debug");
        let original_lib_path = target_debug_dir.join(&original_lib_name);

        if !original_lib_path.exists() {
            self.log("⚠ The original library does not exist yet. Compile the project.");
            return;
        }

        if let Ok(entries) = std::fs::read_dir(&target_debug_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                if file_name.starts_with(&format!("{}{}_hotreload_", prefix, proj_name)) {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let hotreload_lib_name = format!("{}{}_hotreload_{}.{}", prefix, proj_name, timestamp, ext);
        let hotreload_lib_path = target_debug_dir.join(&hotreload_lib_name);

        if let Err(e) = std::fs::copy(&original_lib_path, &hotreload_lib_path) {
            self.log(&format!("❌ DLL copy error: {}", e));
            return;
        }

        unsafe {
            match libloading::Library::new(&hotreload_lib_path) {
                Ok(lib) => {
                    let func: Result<
                        libloading::Symbol<unsafe extern "C" fn() -> *mut en_core::PluginRegistry>,
                        _,
                    > = lib.get(b"en_get_plugin_registry\0");

                    if let Ok(get_registry) = func {
                        let registry_ptr = get_registry();
                        let registry = Box::from_raw(registry_ptr);

                        self.engine.reload_plugins(*registry);
                        self.log("✨ Domain Reload completed! Logic updated.");

                        self.ui_state.active_plugin_lib = Some(lib);
                    } else {
                        self.log("⚠ 'en_get_plugin_registry' not found in library.");
                    }
                }
                Err(e) => {
                    self.log(&format!("❌ Failed to load DLL: {}", e));
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------
// EFRAME UPDATE LOOP
// ----------------------------------------------------------------------------
impl eframe::App for EditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.check_build_status();

        egui::Panel::top("top_bar")
            .frame(en_ui::theme::bar_frame())
            .show_inside(ui, |ui| {
                panels::top_bar::draw(ui, self);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                let mut current_tree =
                    std::mem::replace(&mut self.ui_state.tree, Tree::empty("placeholder"));

                {
                    let mut behavior = EditorBehavior { app: self, frame };

                    // Малюємо структуру панелей (Docking)
                    current_tree.ui(&mut behavior, ui);
                }

                // Повертаємо дерево назад
                self.ui_state.tree = current_tree;
            });

        if self.ui_state.is_playing {
            self.engine.update();
            ui.request_repaint();
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {}
}
