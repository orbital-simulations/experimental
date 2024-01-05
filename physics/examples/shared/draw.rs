use glam::{vec2, DVec2, Vec2};
use macroquad::color::Color;

use physics::{Collision, Engine, Particle, Shape};

pub fn to_macroquad(v: DVec2, size: Vec2) -> Vec2 {
    vec2(size.x / 2.0 + v.x as f32, size.y / 2.0 - v.y as f32)
}

pub fn draw_vec_line(from: DVec2, to: DVec2, thickness: f32, color: Color, size: Vec2) {
    use macroquad::shapes::draw_line;
    let from = to_macroquad(from, size);
    let to = to_macroquad(to, size);
    draw_line(from.x, from.y, to.x, to.y, thickness, color)
}

pub trait Draw {
    fn draw(&self, size: Vec2);
}

impl Draw for Particle {
    fn draw(&self, size: Vec2) {
        use glam::DMat2;
        use macroquad::color::WHITE;
        use macroquad::shapes::draw_circle_lines;
        use Shape::*;
        match self.shape {
            Circle { radius: r } => {
                let pos = to_macroquad(self.pos, size);
                draw_circle_lines(pos.x, pos.y, r as f32, 1.0, WHITE);
                let x = r * DMat2::from_angle(self.angle) * DVec2::X;
                let y = r * DMat2::from_angle(self.angle) * DVec2::Y;
                let pos = self.pos;
                draw_vec_line(pos + x, pos - x, 1.0, WHITE, size);
                draw_vec_line(pos + y, pos - y, 1.0, WHITE, size);
            }
            HalfPlane { normal_angle } => {
                let extent = size.x.max(size.y) as f64;
                let tangent = DVec2::from_angle(normal_angle).perp();
                let from = self.pos + extent * tangent;
                let to = self.pos - extent * tangent;
                draw_vec_line(from, to, 1.0, WHITE, size);
            }
            _ => {
                unimplemented!("Unknown shape {:?}", self.shape)
            }
        }
    }
}

impl Draw for Collision {
    fn draw(&self, size: Vec2) {
        use macroquad::color::RED;
        let contact = &self.contact;
        let pos_inside = contact.pos + contact.separation * contact.normal;
        draw_vec_line(contact.pos, pos_inside, 2.0, RED, size);
    }
}

impl Draw for Engine {
    fn draw(&self, size: Vec2) {
        for p in &self.particles {
            p.draw(size);
        }

        for col in self.detect_collisions() {
            col.draw(size);
        }
    }
}
