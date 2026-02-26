use en::*;

pub mod en;
mod components;
mod macros;
mod sprite_manager;
mod editor;
mod physics;
mod aseprite;
mod systems;

pub const PPU: f32 = 128.0;

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

    if let Ok(bytes) = std::fs::read("Scene.bin") {
        load_scene(&mut world, &bytes);
        for (_id, ren) in world.query_mut::<&mut Render>() {
            ren.cached_sprite = None;
        }
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
            handle_editor_input(
                &mut world, &camera, &mut selected_entity, &mut dragging_entity,
                &mut drag_offset, &mut ctx_menu_world, &mut ctx_menu_screen
            );
        }

        // 4. RENDER WORLD 
        render_world(
            &mut world, &mut sprite_manager, &camera, final_zoom,
            #[cfg(debug_assertions)] show_editor,
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

#[cfg(debug_assertions)]
fn handle_editor_input(
    world: &mut World, camera: &Camera2D, 
    selected: &mut Option<hecs::Entity>, dragging: &mut Option<hecs::Entity>, 
    offset: &mut Vec2, ctx_world: &mut Option<Vec2>, ctx_screen: &mut Option<Vec2>
) {
    let m_pos = mouse_position();
    let mouse_world = camera.screen_to_world(vec2(m_pos.0, m_pos.1));

    if is_mouse_button_pressed(MouseButton::Left) {
        *ctx_world = None; 
        *ctx_screen = None;

        let mut clicked = None;
        for (id, (pos, ren)) in world.query_mut::<(&Pos, &Render)>() {
            let rect = Rect::new(pos.x - ren.w / 2.0, pos.y - ren.h / 2.0, ren.w, ren.h);
            if rect.contains(mouse_world) {
                clicked = Some(id);
                *offset = vec2(pos.x, pos.y) - mouse_world;
            }
        }
        *selected = clicked; 
        *dragging = clicked; 
    }

    if is_mouse_button_down(MouseButton::Left) {
        if let Some(entity) = dragging {
            if let Ok(mut pos) = world.get::<&mut Pos>(*entity) {
                let target_x = mouse_world.x + offset.x;
                let target_y = mouse_world.y + offset.y;

                if is_key_down(KeyCode::LeftShift) {
                    pos.x = (target_x / 16.0).round() * 16.0;
                    pos.y = (target_y / 16.0).round() * 16.0;
                } else {
                    pos.x = target_x;
                    pos.y = target_y;
                }
            }
        }
    }

    if is_mouse_button_released(MouseButton::Left) { *dragging = None; }
    if is_mouse_button_pressed(MouseButton::Right) {
        *ctx_world = Some(mouse_world);
        *ctx_screen = Some(vec2(m_pos.0, m_pos.1));
    }
}

fn render_world(
    world: &mut World, sprites: &mut SpriteManager, camera: &Camera2D, zoom: f32, 
    #[cfg(debug_assertions)] show_editor: bool, 
    #[cfg(debug_assertions)] selected: Option<hecs::Entity>
) {
    #[cfg(debug_assertions)]
    if show_editor {
        let view_size = vec2(screen_width(), screen_height()) / zoom;
        let start_x = ((camera.target.x - view_size.x / 2.0) / PPU).floor() * PPU;
        let end_x = ((camera.target.x + view_size.x / 2.0) / PPU).ceil() * PPU;
        let start_y = ((camera.target.y - view_size.y / 2.0) / PPU).floor() * PPU;
        let end_y = ((camera.target.y + view_size.y / 2.0) / PPU).ceil() * PPU;

        let mut x = start_x;
        while x <= end_x {
            draw_line(x, start_y, x, end_y, 1.0 / zoom, BLUE);
            x += PPU;
        }
        let mut y = start_y;
        while y <= end_y {
            draw_line(start_x, y, end_x, y, 1.0 / zoom, BLUE);
            y += PPU;
        }
    }

    for (_id, (pos, ren)) in world.query_mut::<(&Pos, &mut Render)>() {
        if ren.cached_sprite.is_none() {
            ren.cached_sprite = sprites.sprites.get(ren.s_id.0).cloned();
        }

        if let Some(sprite) = &ren.cached_sprite {
            draw_texture_ex(
                &sprite.texture,
                pos.x - ren.w / 2.0, pos.y - ren.h / 2.0,
                Color::new(ren.r(), ren.g(), ren.b(), ren.a()),
                DrawTextureParams {
                    source: Some(sprite.source_rect),
                    flip_x: ren.flip_x,
                    flip_y: !ren.flip_y,
                    dest_size: Some(vec2(ren.w, ren.h)),
                    ..Default::default()
                }
            );
        }
    }

    #[cfg(debug_assertions)]
    if show_editor {
        for (_id, (pos, col)) in world.query_mut::<(&Pos, &Collider)>() {
            draw_rectangle_lines(pos.x - col.w / 2.0, pos.y - col.h / 2.0, col.w, col.h, 1.0, GREEN);
        }

        if let Some(entity) = selected {
            if let Ok(pos) = world.get::<&Pos>(entity) {
                if let Ok(ren) = world.get::<&Render>(entity) {
                    draw_rectangle_lines(pos.x - ren.w / 2.0 - 2.0, pos.y - ren.h / 2.0 - 2.0, ren.w + 4.0, ren.h + 4.0, 2.0, WHITE);
                }
                if let Ok(col) = world.get::<&Collider>(entity) {
                    draw_rectangle_lines(pos.x - col.w / 2.0, pos.y - col.h / 2.0, col.w, col.h, 2.0, RED);
                }
            }
        }
    }
}