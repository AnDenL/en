use bevy_ecs::prelude::Component;
use glam::Mat4;

#[derive(Component)]
pub struct Camera2D {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub znear: f32,
    pub zfar: f32,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            left: -640.0, right: 640.0, bottom: -360.0, top: 360.0,
            znear: -1.0, zfar: 1.0,
            x: 0.0, y: 0.0, scale: 1.0,
        }
    }
}

impl Camera2D {
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left, self.right, self.bottom, self.top, self.znear, self.zfar,
        )
    }

    pub fn update_aspect_ratio(&mut self, width: f32, height: f32) {
        let aspect_ratio = width / height;
        
        let half_height = 360.0 * self.scale; 
        let half_width = half_height * aspect_ratio;

        self.left = self.x - half_width;
        self.right = self.x + half_width;
        self.bottom = self.y - half_height;
        self.top = self.y + half_height;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera2D) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}