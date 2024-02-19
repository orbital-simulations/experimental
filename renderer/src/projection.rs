use glam::Mat4;

#[derive(Debug)]
pub enum Projection {
    Perspective(PerspectiveProjection),
    Ortographic(OrtographicProjection),
}

#[derive(Debug)]
pub struct OrtographicProjection {
    width: f32,
    height: f32,
    depth: f32,
    scale_factor: f32,
}

impl OrtographicProjection {
    pub fn new(width: f32, height: f32, depth: f32, scale_factor: f32) -> Self {
        Self {
            width,
            height,
            depth,
            scale_factor,
        }
    }
}

pub trait ProjectionManipulation {
    fn resize(&mut self, width: f32, height: f32);
    fn scale(&mut self, scale_factor: f32);
    fn make_projection_matrix(&self) -> Mat4;
}

impl ProjectionManipulation for OrtographicProjection {
    fn make_projection_matrix(&self) -> Mat4 {
        let half_width = self.width / (2. * self.scale_factor);
        let half_height = self.height / (2. * self.scale_factor);
        let half_depth = self.depth / 2.;
        glam::Mat4::orthographic_rh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            -half_depth,
            half_depth,
        )
    }

    fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    fn scale(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }
}

#[derive(Debug)]
pub struct PerspectiveProjection {
    aspect: f32,
    fovy: f32, // In radians
    znear: f32,
    zfar: f32,
    scale: f32,
}

impl PerspectiveProjection {
    pub fn new(width: f32, height: f32, fovy: f32, znear: f32, zfar: f32, scale: f32) -> Self {
        let aspect = width / height;
        Self {
            aspect,
            fovy,
            znear,
            zfar,
            scale,
        }
    }
}

impl ProjectionManipulation for PerspectiveProjection {
    fn make_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy / self.scale, self.aspect, self.znear, self.zfar)
    }

    fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    fn scale(&mut self, scale_factor: f32) {
        self.scale = scale_factor;
    }
}

impl ProjectionManipulation for Projection {
    fn make_projection_matrix(&self) -> Mat4 {
        match self {
            Projection::Perspective(p) => p.make_projection_matrix(),
            Projection::Ortographic(p) => p.make_projection_matrix(),
        }
    }

    fn resize(&mut self, width: f32, height: f32) {
        match self {
            Projection::Perspective(p) => p.resize(width, height),
            Projection::Ortographic(p) => p.resize(width, height),
        }
    }

    fn scale(&mut self, scale_factor: f32) {
        match self {
            Projection::Perspective(p) => p.scale(scale_factor),
            Projection::Ortographic(p) => p.scale(scale_factor),
        }
    }
}
