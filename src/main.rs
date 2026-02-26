use en::*;

pub mod en;
mod components;
mod macros;
mod sprite_manager;
mod editor;
mod physics;
mod aseprite;
mod systems;
mod render;

fn window_conf() -> Conf {
    Conf {
        window_title: "En game".to_owned(),
        window_width: 1920,
        window_height: 1080,
        sample_count: 0,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = World::new();
    let mut sprite_manager = SpriteManager::new();
    sprite_manager.load_all("assets/sprites").await;

    if let Ok(bytes) = macroquad::file::load_file("Scene.bin").await {
        load_scene(&mut world, &bytes);
        for (_id, ren) in world.query_mut::<&mut Render>() {
            ren.cached_sprite = None;
        }
        println!("Scene loaded successfully!");
    } else {
        println!("Scene.bin not found or failed to load.");
    }

    let mut is_paused = false;
    let mut camera_zoom = 1.0;
    let mut camera_free_pos = vec2(0.0, 0.0);

    #[cfg(debug_assertions)] let mut selected_entity: Option<hecs::Entity> = None;
    #[cfg(debug_assertions)] let mut dragging_entity: Option<hecs::Entity> = None;
    #[cfg(debug_assertions)] let mut drag_offset = vec2(0.0, 0.0);
    #[cfg(debug_assertions)] let mut show_editor = true;
    #[cfg(debug_assertions)] let mut block_editor_input = false;
    #[cfg(debug_assertions)] let mut ctx_menu_world: Option<Vec2> = None;
    #[cfg(debug_assertions)] let mut ctx_menu_screen: Option<Vec2> = None;
    #[cfg(debug_assertions)] let mut brush_mode = false;

    loop {
        clear_background(DARKBLUE);
        let dt = get_frame_time().min(0.1);

        // 1. UPDATE 
        if !is_paused {
            systems::run_all_systems(&mut systems::SysCtx { world: &mut world, sprites: &mut sprite_manager, dt });
            physics::update_physics(&mut world, dt);
        }

        // 2. CAMERA 
        let (final_cam_pos, final_zoom) = update_camera_logic(
            &world, dt, &mut camera_free_pos, &mut camera_zoom, 
            #[cfg(debug_assertions)] show_editor, 
            #[cfg(debug_assertions)] block_editor_input
        );

        let camera = Camera2D {
            target: final_cam_pos,
            zoom: vec2(1.0 / screen_width() * 2.0 * final_zoom, -1.0 / screen_height() * 2.0 * final_zoom),
            ..Default::default()
        };
        set_camera(&camera);

        // 3. EDITOR INPUT 
        #[cfg(debug_assertions)]
        if show_editor && !block_editor_input {
            editor::handle_editor_input(
                &mut world, &camera, &mut selected_entity, &mut dragging_entity,
                &mut drag_offset, &mut ctx_menu_world, &mut ctx_menu_screen, &mut brush_mode
            );
        }

        // 4. RENDER WORLD 
        render::render_world(
            &mut world, &mut sprite_manager, &camera, final_zoom,
            #[cfg(debug_assertions)] show_editor,
            #[cfg(debug_assertions)] brush_mode,
            #[cfg(debug_assertions)] selected_entity
        );

        set_default_camera();

        // 5. RENDER UI 
        #[cfg(debug_assertions)]
        {
            if is_key_pressed(KeyCode::Tab) { show_editor = !show_editor; }
            if show_editor {
                editor::draw_editor(
                    &mut world, &mut selected_entity, &mut is_paused, 
                    &mut block_editor_input, &mut ctx_menu_world, 
                    &mut ctx_menu_screen, &sprite_manager 
                );
                egui_macroquad::draw();
            }
        }

        next_frame().await;
    }
}

fn update_camera_logic(
    world: &World, dt: f32, free_pos: &mut Vec2, free_zoom: &mut f32, 
    #[cfg(debug_assertions)] show_editor: bool, 
    #[cfg(debug_assertions)] block_input: bool
) -> (Vec2, f32) {
    let mut game_pos = vec2(0.0, 0.0);
    let mut game_zoom = 1.0;
    
    for (_id, (pos, cam)) in world.query::<(&Pos, &CameraAnchor)>().iter() {
        game_pos = vec2(pos.x, pos.y);
        game_zoom = cam.zoom;
    }

    #[cfg(debug_assertions)]
    if show_editor {
        let diff = if is_key_down(KeyCode::LeftShift) { 1200.0 } else { 400.0 } * dt / *free_zoom;
        if is_key_down(KeyCode::D) { free_pos.x += diff; }
        if is_key_down(KeyCode::A) { free_pos.x -= diff; }
        if is_key_down(KeyCode::W) { free_pos.y += diff; }
        if is_key_down(KeyCode::S) { free_pos.y -= diff; }

        if !block_input {
            let (_, mouse_y) = mouse_wheel();
            *free_zoom = (*free_zoom + mouse_y * 0.15).clamp(0.1, 7.0);
        }
        return (*free_pos, *free_zoom);
    }

    *free_pos = game_pos;
    *free_zoom = game_zoom;
    (game_pos, game_zoom)
}