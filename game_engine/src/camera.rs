use glam::{vec3, Mat4, Vec3};
use std::f32::consts::FRAC_PI_2;
use winit::{dpi::PhysicalPosition, event::MouseScrollDelta, keyboard::KeyCode};

use crate::inputs::InputKeys;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    position: Vec3,
    yaw: f32,   // In radians
    pitch: f32, // In radians
}

impl Camera {
    pub fn new<V: Into<Vec3>, Y: Into<f32>, P: Into<f32>>(position: V, yaw: Y, pitch: P) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        Mat4::look_to_rh(
            self.position,
            vec3(sin_yaw * cos_pitch, cos_yaw * cos_pitch, sin_pitch),
            //vec3(sin_pitch * sin_yaw, sin_pitch, sin_pitch * cos_yaw).normalize(),
            vec3(0., 0., 1.),
        )
        //Mat4::IDENTITY
    }
}
#[derive(Debug)]
pub struct CameraController {
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * 0.5,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32, input_keys: &InputKeys) {
        let mut forward_backward: f32 = 0.;
        let mut left_rigth: f32 = 0.;
        let mut up_down: f32 = 0.;
        if input_keys.is_key_pressed(KeyCode::KeyW) {
            forward_backward += 1.;
        }
        if input_keys.is_key_pressed(KeyCode::KeyS) {
            forward_backward -= 1.;
        }
        if input_keys.is_key_pressed(KeyCode::KeyA) {
            left_rigth += 1.;
        }
        if input_keys.is_key_pressed(KeyCode::KeyD) {
            left_rigth -= 1.;
        }
        if input_keys.is_key_pressed(KeyCode::Space) {
            up_down += 1.;
        }
        if input_keys.is_key_pressed(KeyCode::ShiftLeft) {
            up_down -= 1.;
        }

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vec3::new(yaw_sin, yaw_cos, 0.0).normalize();
        let right = Vec3::new(-yaw_cos, yaw_sin, 0.0).normalize();
        camera.position += forward * forward_backward * self.speed * dt;
        camera.position += right * left_rigth * self.speed * dt;

        // TODO: Fix zoom, it probably is still not translated to x-y horizontal
        // plane.
        //
        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        let scrollward = Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.z += up_down * self.speed * dt;

        // Rotate
        camera.yaw -= self.rotate_horizontal * self.sensitivity * dt;
        camera.pitch -= -self.rotate_vertical * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -SAFE_FRAC_PI_2 {
            camera.pitch = -SAFE_FRAC_PI_2;
        } else if camera.pitch > SAFE_FRAC_PI_2 {
            camera.pitch = SAFE_FRAC_PI_2;
        }
    }
}
