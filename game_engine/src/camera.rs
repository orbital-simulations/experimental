use glam::{vec3, Mat4, Vec3};
use std::f32::consts::FRAC_PI_2;
use winit::{
    dpi::PhysicalPosition,
    event::{MouseButton, MouseScrollDelta},
    keyboard::KeyCode,
};

use crate::inputs::Inputs;

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
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * 0.5,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32, inputs: &Inputs) {
        let mut forward_backward: f32 = 0.;
        let mut left_rigth: f32 = 0.;
        let mut up_down: f32 = 0.;
        if inputs.is_key_pressed(KeyCode::KeyW) {
            forward_backward += 1.;
        }
        if inputs.is_key_pressed(KeyCode::KeyS) {
            forward_backward -= 1.;
        }
        if inputs.is_key_pressed(KeyCode::KeyA) {
            left_rigth += 1.;
        }
        if inputs.is_key_pressed(KeyCode::KeyD) {
            left_rigth -= 1.;
        }
        if inputs.is_key_pressed(KeyCode::Space) {
            up_down += 1.;
        }
        if inputs.is_key_pressed(KeyCode::ShiftLeft) {
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
        if inputs.is_button_pressed(MouseButton::Left) {
            camera.yaw -= inputs.cursor_delta.map(|v| v.0).unwrap_or(0.) * self.sensitivity * dt;
            camera.pitch -= inputs.cursor_delta.map(|v| v.1).unwrap_or(0.) * self.sensitivity * dt;
        }

        // Keep the camera's angle from going too high/low.
        camera.pitch = camera.pitch.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);
    }
}
