use core::panic;
use std::ops::Mul;

use bytemuck::{Pod, Zeroable};
use glam::{Affine3A, EulerRot, Mat4, Quat, Vec3, Vec4};

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    translate: Vec3,
    rotate: Quat,
    scale: f32,
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        translate: Vec3::ZERO,
        scale: 1.0,
        rotate: Quat::IDENTITY,
    };

    pub fn from_translation(position: &Vec3) -> Self {
        Self {
            translate: *position,
            scale: 1.0,
            rotate: Quat::IDENTITY,
        }
    }

    pub fn from_rotation(rotation: &Quat) -> Self {
        Self {
            translate: Vec3::ZERO,
            scale: 1.0,
            rotate: *rotation,
        }
    }

    pub fn from_rotation_euler(rotation: &Vec3) -> Self {
        Self {
            translate: Vec3::ZERO,
            scale: 1.0,
            rotate: Quat::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z),
        }
    }

    pub fn from_rotation_x(rotation: f32) -> Self {
        Self {
            translate: Vec3::ZERO,
            scale: 1.0,
            rotate: Quat::from_rotation_x(rotation),
        }
    }

    pub fn from_rotation_y(rotation: f32) -> Self {
        Self {
            translate: Vec3::ZERO,
            scale: 1.0,
            rotate: Quat::from_rotation_y(rotation),
        }
    }

    pub fn from_rotation_z(rotation: f32) -> Self {
        Self {
            translate: Vec3::ZERO,
            scale: 1.0,
            rotate: Quat::from_rotation_z(rotation),
        }
    }

    pub fn from_scale(scale: f32) -> Self {
        Self {
            translate: Vec3::ZERO,
            scale,
            rotate: Quat::IDENTITY,
        }
    }

    pub fn from_translation_rotation(position: &Vec3, rotation: &Quat) -> Self {
        Self {
            translate: *position,
            scale: 1.0,
            rotate: *rotation,
        }
    }

    pub fn from_translation_rotation_scale(position: &Vec3, rotation: &Quat, scale: f32) -> Self {
        Self {
            translate: *position,
            scale,
            rotate: *rotation,
        }
    }

    pub fn from_translation_rotation_euler(position: &Vec3, rotation: &Vec3) -> Self {
        Self {
            translate: *position,
            scale: 1.0,
            rotate: Quat::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z),
        }
    }

    pub fn from_translation_rotation_x(position: &Vec3, rotation: f32) -> Self {
        Self {
            translate: *position,
            scale: 1.0,
            rotate: Quat::from_rotation_x(rotation),
        }
    }

    pub fn from_translation_rotation_y(position: &Vec3, rotation: f32) -> Self {
        Self {
            translate: *position,
            scale: 1.0,
            rotate: Quat::from_rotation_y(rotation),
        }
    }

    pub fn from_translation_rotation_z(position: &Vec3, rotation: f32) -> Self {
        Self {
            translate: *position,
            scale: 1.0,
            rotate: Quat::from_rotation_z(rotation),
        }
    }

    pub fn from_columns(columns: &[[f32; 4]; 4]) -> Self {
        let transform = Mat4::from_cols(
         Vec4::from_array(columns[0]),
         Vec4::from_array(columns[1]),
         Vec4::from_array(columns[2]),
         Vec4::from_array(columns[3])
        );

        let (scale, rotate, translate) = transform.to_scale_rotation_translation();
        if scale[0] != scale[1] && scale[0] != scale[2] {
            panic!("scale needs to be uniform `Vec3(n, n, n)` where `n` is \
                scalar scale. The scale actually is: {scale}");
        }
        Self {
            translate,
            scale: scale[0],
            rotate,
        }
    }

    pub fn set_translation(&mut self, translation: &Vec3) {
        self.translate = *translation;
    }

    pub fn set_rotation(&mut self, rotation: &Quat) {
        self.rotate = *rotation;
    }

    /// Rotation set as euclidian vector.
    pub fn set_rotation_euler(&mut self, rotation: &Vec3) {
        self.rotate = Quat::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform::IDENTITY
    }
}

impl Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        Transform {
            translate: self.translate + self.rotate.mul_vec3(rhs.translate * self.scale),
            scale: self.scale * rhs.scale,
            rotate: self.rotate * rhs.rotate,
        }
    }
}

impl<'a> Mul<&'a Transform> for &'a Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        Transform {
            translate: self.translate + self.rotate.mul_vec3(rhs.translate * self.scale),
            scale: self.scale * rhs.scale,
            rotate: self.rotate * rhs.rotate,
        }
    }
}

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C, packed)]
pub struct TransformGpu([f32; 12]);

impl From<Transform> for TransformGpu {
    fn from(value: Transform) -> Self {
        TransformGpu(
            Affine3A::from_scale_rotation_translation(
                Vec3::splat(value.scale),
                value.rotate,
                value.translate,
            )
            .to_cols_array(),
        )
    }
}

impl<'a> From<&'a Transform> for TransformGpu {
    fn from(value: &'a Transform) -> Self {
        TransformGpu(
            Affine3A::from_scale_rotation_translation(
                Vec3::splat(value.scale),
                value.rotate,
                value.translate,
            )
            .to_cols_array(),
        )
    }
}

impl TransformGpu {
    pub fn vertex_attributes(
        x_location: wgpu::ShaderLocation,
        y_location: wgpu::ShaderLocation,
        z_location: wgpu::ShaderLocation,
        w_location: wgpu::ShaderLocation,
    ) -> Vec<wgpu::VertexAttribute> {
        vec![
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: x_location,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: wgpu::VertexFormat::Float32x3.size(),
                shader_location: y_location,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: wgpu::VertexFormat::Float32x3.size() * 2,
                shader_location: z_location,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: wgpu::VertexFormat::Float32x3.size() * 3,
                shader_location: w_location,
            },
        ]
    }
}
