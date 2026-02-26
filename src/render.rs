use hecs::World;
use macroquad::prelude::*;
use crate::components::*;
use crate::sprite_manager::SpriteManager;

pub const PPU: f32 = 128.0;

pub fn render_world(
    world: &mut World, sprites: &mut SpriteManager, camera: &Camera2D, zoom: f32, 
    #[cfg(debug_assertions)] show_editor: bool, 
    #[cfg(debug_assertions)] brush_mode: bool,
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

    let view_size = vec2(screen_width(), screen_height()) / zoom;
    
    let cam_rect = Rect::new(
        camera.target.x - view_size.x / 2.0 - 100.0,
        camera.target.y - view_size.y / 2.0 - 100.0,
        view_size.x + 200.0,
        view_size.y + 200.0,
    );

    for (_id, (pos, tm)) in world.query_mut::<(&Pos, &TileMap)>() {
        let offset_x = (tm.width as f32 * tm.tile_size) / 2.0;
        let offset_y = (tm.height as f32 * tm.tile_size) / 2.0;
        
        let map_start_x = pos.x - offset_x;
        let map_start_y = pos.y - offset_y;

        let view_size = vec2(screen_width(), screen_height()) / zoom;
        
        let start_x = (((camera.target.x - view_size.x / 2.0) - map_start_x) / tm.tile_size).floor() as i32;
        let end_x = (((camera.target.x + view_size.x / 2.0) - map_start_x) / tm.tile_size).ceil() as i32;
        let start_y = (((camera.target.y - view_size.y / 2.0) - map_start_y) / tm.tile_size).floor() as i32;
        let end_y = (((camera.target.y + view_size.y / 2.0) - map_start_y) / tm.tile_size).ceil() as i32;

        for y in start_y.max(0)..=end_y.min((tm.height - 1) as i32) {
            for x in start_x.max(0)..=end_x.min((tm.width - 1) as i32) {
                let idx = (y as usize) * tm.width + (x as usize);
                let sprite_idx = tm.tiles[idx];
                
                if sprite_idx != 0 {
                    if let Some(sprite) = sprites.sprites.get(&sprite_idx) {
                        draw_texture_ex(
                            &sprite.texture,
                            map_start_x + (x as f32) * tm.tile_size,
                            map_start_y + (y as f32) * tm.tile_size,
                            WHITE,
                            DrawTextureParams {
                                source: Some(sprite.source_rect),
                                flip_y: true,
                                dest_size: Some(vec2(tm.tile_size, tm.tile_size)),
                                ..Default::default()
                            }
                        );
                    }
                }
            }
        }
    }

    for (_id, (pos, ren)) in world.query_mut::<(&Pos, &mut Render)>() {
        let sprite_rect = Rect::new(pos.x - ren.w / 2.0, pos.y - ren.h / 2.0, ren.w, ren.h);
        
        if !cam_rect.overlaps(&sprite_rect) {
            continue; 
        }
        if ren.cached_sprite.is_none() {
            ren.cached_sprite = sprites.sprites.get(&ren.s_id.0).cloned();
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
        if brush_mode {
            let m_pos = mouse_position();
            let mouse_world = camera.screen_to_world(vec2(m_pos.0, m_pos.1));
            
            if let Some(entity) = selected {
                if let Ok(tm) = world.get::<&TileMap>(entity) {
                    if let Ok(pos) = world.get::<&Pos>(entity) {
                        let offset_x = (tm.width as f32 * tm.tile_size) / 2.0;
                        let offset_y = (tm.height as f32 * tm.tile_size) / 2.0;
                        let map_start_x = pos.x - offset_x;
                        let map_start_y = pos.y - offset_y;

                        let gx = ((mouse_world.x - map_start_x) / tm.tile_size).floor();
                        let gy = ((mouse_world.y - map_start_y) / tm.tile_size).floor();

                        draw_rectangle_lines(
                            map_start_x + gx * tm.tile_size, 
                            map_start_y + gy * tm.tile_size, 
                            tm.tile_size, tm.tile_size, 2.0 / zoom, YELLOW
                        );
                    }
                }
            }
        }
    }
}