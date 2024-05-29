use glam::{Mat4, Vec2};

#[derive(Clone, Debug)]
pub struct Perspective {
    pub fovy: f32, // In radians
    pub znear: f32,
    pub zfar: f32,
    pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct Orthographic {
    pub depth: f32,
    pub scale: f32,
}

#[derive(Clone, Debug)]
pub enum CameraProjection {
    Perspective(Perspective),
    Orthographic(Orthographic),
}

impl CameraProjection {
    pub fn make_projection_matrix(&self, size: Vec2) -> Mat4 {
        match self {
            CameraProjection::Perspective(Perspective {
                fovy,
                znear,
                zfar,
                scale,
            }) => {
                let aspect = size.x / size.y;
                Mat4::perspective_rh(fovy / scale, aspect, *znear, *zfar)
            }
            CameraProjection::Orthographic(Orthographic { depth, scale }) => {
                let half_width = size.x / (2. * scale);
                let half_height = size.y / (2. * scale);
                let half_depth = depth / 2.;
                glam::Mat4::orthographic_rh(
                    -half_width,
                    half_width,
                    -half_height,
                    half_height,
                    -half_depth,
                    half_depth,
                )
            }
        }
    }

    pub fn set_scale(&mut self, scale: f32) {
        match self {
            CameraProjection::Perspective(perspective) => perspective.scale = scale,
            CameraProjection::Orthographic(orthographic) => orthographic.scale = scale,
        }
    }
}
