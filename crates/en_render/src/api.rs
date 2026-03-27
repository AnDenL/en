use crate::batcher::SpriteBatcher;
use crate::config::RenderSettings;
use crate::context::RenderContext;
use crate::texture::GpuTexture;
use crate::types::{InstanceData, RenderBatch};
use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {
    context: RenderContext,
    settings: RenderSettings,
    sprite_pass: crate::pass::sprite_pass::SpritePass,
    pub default_white_bind_group: wgpu::BindGroup,
}
impl Renderer {
    pub async fn new(window: Arc<Window>, settings: RenderSettings) -> Result<Self, &'static str> {
        let context = RenderContext::new(window, &settings).await?;
        let sprite_pass =
            crate::pass::sprite_pass::SpritePass::new(&context.device, context.render_format);
        let default_white_bind_group = GpuTexture::create_default_white_bind_group(
            &context.device,
            &context.queue,
            &sprite_pass.texture_bind_group_layout,
        );

        Ok(Self {
            context,
            settings,
            sprite_pass,
            default_white_bind_group,
        })
    }

    pub fn new_headless(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        format: wgpu::TextureFormat,
        settings: RenderSettings,
    ) -> Self {
        let context = RenderContext::new_headless(device.clone(), queue.clone(), format);
        let sprite_pass = crate::pass::sprite_pass::SpritePass::new(&context.device, format);
        let default_white_bind_group = GpuTexture::create_default_white_bind_group(
            &context.device,
            &context.queue,
            &sprite_pass.texture_bind_group_layout,
        );

        Self {
            context,
            settings,
            sprite_pass,
            default_white_bind_group,
        }
    }

    pub fn texture_layout(&self) -> &wgpu::BindGroupLayout {
        &self.sprite_pass.texture_bind_group_layout
    }

    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.context.device
    }
    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.context.queue
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.context.resize(width, height);
    }

    pub fn update_settings(&mut self, new_settings: RenderSettings) {
        if self.settings.vsync != new_settings.vsync {
            self.context.set_vsync(new_settings.vsync);
        }
        self.settings = new_settings;
    }

    pub fn update_camera(&mut self, view_proj_matrix: [[f32; 4]; 4]) {
        self.sprite_pass
            .update_camera_buffer(&self.context.queue, view_proj_matrix);
    }

    /// Основний метод рендера для гри. Тепер приймає Batcher напряму для зручності.
    pub fn render(&mut self, batcher: &mut SpriteBatcher) -> Result<(), wgpu::SurfaceError> {
        batcher.finish(); // Закриваємо останній батч автоматично

        let surface = self
            .context
            .surface
            .as_ref()
            .expect("Renderer has no surface!");
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_internal(batcher.batches(), batcher.instances(), &view);

        output.present();
        Ok(())
    }

    pub fn render_to_view(&mut self, batcher: &mut SpriteBatcher, view: &wgpu::TextureView) {
        batcher.finish();
        self.render_internal(batcher.batches(), batcher.instances(), view);
    }

    fn render_internal(
        &mut self,
        batches: &[RenderBatch],
        instances: &[InstanceData],
        target_view: &wgpu::TextureView,
    ) {
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Main Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.settings.clear_color[0] as f64,
                            g: self.settings.clear_color[1] as f64,
                            b: self.settings.clear_color[2] as f64,
                            a: self.settings.clear_color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });

            // Передаємо в пас і батчі, і єдиний загальний масив інстансів
            self.sprite_pass.draw(
                &self.context.device,
                &self.context.queue,
                &mut render_pass,
                batches,
                instances,
            );
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn render_format(&self) -> wgpu::TextureFormat {
        self.context.render_format
    }
}
