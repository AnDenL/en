use crate::app::EditorApp;
use eframe::egui;

pub fn draw(ui: &mut egui::Ui, app: &mut EditorApp, frame: &mut eframe::Frame) {
    let size = ui.available_size();
    let width = size.x.max(1.0) as u32;
    let height = size.y.max(1.0) as u32;

    let needs_recreate = app.ui_state.viewport_texture.as_ref().map_or(true, |tex| {
        tex.size().width != width || tex.size().height != height
    });

    if needs_recreate {
        let wgpu_state = frame
            .wgpu_render_state()
            .expect("WGPU is not enabled in eframe!");

        if let Some(old_id) = app.ui_state.viewport_texture_id {
            wgpu_state.renderer.write().free_texture(&old_id);
        }

        let texture = app
            .engine
            .renderer
            .device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Viewport Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Беремо формат напряму з нашого рендера
                format: app.engine.renderer.render_format(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let id = wgpu_state.renderer.write().register_native_texture(
            app.engine.renderer.device(),
            &view,
            wgpu::FilterMode::Linear,
        );

        app.ui_state.viewport_texture = Some(texture);
        app.ui_state.viewport_texture_id = Some(id);
    }

    if let (Some(texture), Some(id)) = (
        &app.ui_state.viewport_texture,
        app.ui_state.viewport_texture_id,
    ) {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let camera_id = ui.id().with("editor_viewport_camera");
        let cam_state = ui.data_mut(|d| {
            d.get_temp::<(f32, f32, f32)>(camera_id)
                .unwrap_or((0.0, 0.0, 1.0))
        });

        let mut cam_x = cam_state.0;
        let mut cam_y = cam_state.1;
        let mut cam_zoom = cam_state.2;

        let hw = (width as f32 / 2.0) * cam_zoom;
        let hh = (height as f32 / 2.0) * cam_zoom;

        let view_proj =
            glam::Mat4::orthographic_rh(cam_x - hw, cam_x + hw, cam_y - hh, cam_y + hh, -1.0, 1.0);

        app.engine
            .renderer
            .update_camera(view_proj.to_cols_array_2d());

        app.engine.render_editor_view(&view);

        let image =
            egui::Image::new(egui::load::SizedTexture::new(id, size)).sense(egui::Sense::drag());
        let response = ui.add(image);

        let mut camera_changed = false;

        if response.dragged_by(egui::PointerButton::Secondary) {
            let delta = response.drag_delta();
            cam_x -= delta.x * cam_zoom;
            cam_y += delta.y * cam_zoom;
            camera_changed = true;
        }

        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let zoom_speed = 0.001;
                cam_zoom *= 1.0 - (scroll * zoom_speed);
                cam_zoom = cam_zoom.clamp(0.01, 100.0);
                camera_changed = true;
            }
        }

        if camera_changed {
            ui.data_mut(|d| d.insert_temp(camera_id, (cam_x, cam_y, cam_zoom)));
        }
    }
}
