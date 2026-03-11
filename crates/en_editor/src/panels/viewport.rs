use eframe::egui;
use crate::app::EditorApp;

/// Draws the Viewport Tab (The actual Game Engine Render)
pub fn draw(ui: &mut egui::Ui, app: &mut EditorApp, frame: &mut eframe::Frame) {
    // 1. Calculate the available space in this specific dock tab
    let size = ui.available_size();
    let width = size.x.max(1.0) as u32;
    let height = size.y.max(1.0) as u32;

    // 2. Check if we need to resize the wgpu texture
    let needs_recreate = app.ui_state.viewport_texture.as_ref().map_or(true, |tex| {
        tex.size().width != width || tex.size().height != height
    });

    // 3. Recreate the texture if the tab was resized
    if needs_recreate {
        let wgpu_state = frame.wgpu_render_state().expect("WGPU is not enabled in eframe!");
        
        if let Some(old_id) = app.ui_state.viewport_texture_id {
            wgpu_state.renderer.write().free_texture(&old_id);
        }
        
        // We use the format that the renderer pipeline was created with
        let texture = app.engine.renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Viewport Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Fallback to Bgra8Unorm if you haven't stored the format in Renderer yet
            format: wgpu::TextureFormat::Bgra8Unorm, 
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Register the native wgpu texture so egui can draw it
        let id = wgpu_state.renderer.write().register_native_texture(
            &*app.engine.renderer.device,
            &view,
            wgpu::FilterMode::Linear,
        );

        app.engine.renderer.camera.update_aspect_ratio(width as f32, height as f32);
        app.engine.renderer.update_camera_buffer();

        app.ui_state.viewport_texture = Some(texture);
        app.ui_state.viewport_texture_id = Some(id);
    }

    // 4. Render the Game Engine to the texture
    if let (Some(texture), Some(id)) = (&app.ui_state.viewport_texture, app.ui_state.viewport_texture_id) {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 🔥 THE MAGIC HAPPENS HERE 🔥
        // We no longer parse JSON! We just ask the ECS for the current state!
        let instances = app.engine.get_render_instances();
        
        // Tell the engine to draw these instances into our Viewport texture
        app.engine.renderer.render_to_view(&instances, &view);
        
        // Display the texture in egui
        let image = egui::Image::new(egui::load::SizedTexture::new(id, size))
            .sense(egui::Sense::drag());
            
        let response = ui.add(image);
        let mut camera_changed = false;

        // --- EDITOR CAMERA CONTROLS ---
        
        // Right click + Drag to pan the camera
        if response.dragged_by(egui::PointerButton::Secondary) {
            let delta = response.drag_delta();
            let world_unit_per_pixel = (360.0 * 2.0 * app.engine.renderer.camera.scale) / size.y;

            app.engine.renderer.camera.x -= delta.x * world_unit_per_pixel;
            app.engine.renderer.camera.y += delta.y * world_unit_per_pixel;
            camera_changed = true;
        }

        // Scroll to zoom
        if response.hovered() {
            // We use ui.input() because ctx.input() requires ctx which we don't pass here
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let zoom_speed = 0.001;
                app.engine.renderer.camera.scale *= 1.0 - (scroll * zoom_speed);
                app.engine.renderer.camera.scale = app.engine.renderer.camera.scale.clamp(0.01, 100.0);
                camera_changed = true;
            }
        }

        if camera_changed {
            app.engine.renderer.camera.update_aspect_ratio(size.x, size.y);
            app.engine.renderer.update_camera_buffer();
        }
    }
}