use glam::{Mat4, Vec3, Vec4};

pub struct Transform {
    translate: Vec3,
    scale: Vec3,
    rotate: Vec3,
}

impl Transform {
    pub fn from_translation(position: &Vec3) -> Self {
        Self {
            translate: *position,
            scale: Vec3::ONE,
            rotate: Vec3::ZERO,
        }
    }

    pub fn from_rotation(rotation: &Vec3) -> Self {
        Self {
            translate: Vec3::ZERO,
            scale: Vec3::ONE,
            rotate: *rotation,
        }
    }

    pub fn from_scale(scale: &Vec3) -> Self {
        Self {
            translate: Vec3::ZERO,
            scale: *scale,
            rotate: Vec3::ZERO,
        }
    }

    pub fn set_translation(&mut self, translation: &Vec3) {
        self.translate = *translation;
    }

    pub fn set_rotation(&mut self, rotation: &Vec3) {
        self.rotate = *rotation;
    }

    pub fn set_scale(&mut self, scale: &Vec3) {
        self.scale = *scale;
    }

    pub fn matrix(&self) -> Mat4 {
        Mat4::from_cols(
            Vec4::new(self.scale.x, 0.0, 0.0, 0.0),
            Vec4::new(0.0, self.scale.y, 0.0, 0.0),
            Vec4::new(0.0, 0.0, self.scale.z, 0.0),
            Vec4::new(self.translate.x, self.translate.y, self.translate.z, 1.0),
        ) * Mat4::from_rotation_z(self.rotate.z)
    }
}
