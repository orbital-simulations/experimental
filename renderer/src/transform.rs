use bytemuck::{Pod, Zeroable};
use glam::{Affine3A, EulerRot, Quat, Vec3};

#[derive(Clone)]
pub struct Transform {
    translate: Vec3,
    scale: Vec3,
    rotate: Quat,
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
            translate: Vec3::ZERO,
            scale: Vec3::ONE,
            rotate: Quat::IDENTITY,
        };

    pub fn from_translation(position: &Vec3) -> Self {
        Self {
            translate: *position,
            scale: Vec3::ONE,
            rotate: Quat::IDENTITY,
        }
    }

    pub fn from_rotation(rotation: &Quat) -> Self {
        Self {
            translate: Vec3::ZERO,
            scale: Vec3::ONE,
            rotate: *rotation,
        }
    }

    pub fn from_rotation_euler(rotation: &Vec3) -> Self {
        Self {
            translate: Vec3::ZERO,
            scale: Vec3::ONE,
            rotate: Quat::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z),
        }
    }

    pub fn from_scale(scale: &Vec3) -> Self {
        Self {
            translate: Vec3::ZERO,
            scale: *scale,
            rotate: Quat::IDENTITY,
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

    pub fn set_scale(&mut self, scale: &Vec3) {
        self.scale = *scale;
    }

    pub fn to_world(&self) -> WorldTransform {
        WorldTransform(Affine3A::from_scale_rotation_translation(self.scale, self.rotate, self.translate))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WorldTransform(Affine3A);

impl WorldTransform {
    pub fn gpu(&self) -> WorldTransformGpuRepresentation {
        WorldTransformGpuRepresentation(self.0.to_cols_array())
    }

}

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C, packed)]
pub struct WorldTransformGpuRepresentation([f32; 12]);

impl WorldTransformGpuRepresentation {
    pub fn vertex_attributes(x_location: wgpu::ShaderLocation, y_location: wgpu::ShaderLocation, z_location: wgpu::ShaderLocation, w_location: wgpu::ShaderLocation) -> Vec<wgpu::VertexAttribute>{
        vec![
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: x_location
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: wgpu::VertexFormat::Float32x3.size(),
                shader_location: y_location
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: wgpu::VertexFormat::Float32x3.size() * 2,
                shader_location: z_location
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: wgpu::VertexFormat::Float32x3.size() * 3,
                shader_location: w_location
            },

        ]
    }
}
