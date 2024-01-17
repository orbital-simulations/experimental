use glam::{DMat2, DVec2};
use macroquad::{
    color::{Color, BLUE, GREEN, RED, WHITE},
    shapes::{draw_circle_lines, draw_line},
};

use physics::{
    constraint::{CollisionConstraint, Constraint},
    geometry::{self},
    Engine, Particle, Shape,
};

pub fn draw_line_vec(from: DVec2, to: DVec2, thickness: f64, color: Color) {
    let from = from.as_vec2();
    let to = to.as_vec2();
    draw_line(from.x, from.y, to.x, to.y, thickness as f32, color);
}

pub fn draw_circle_lines_vec(pos: DVec2, radius: f64, thickness: f64, color: Color) {
    let pos = pos.as_vec2();
    draw_circle_lines(pos.x, pos.y, radius as f32, thickness as f32, color);
}

pub trait Draw {
    fn draw(&self);
}

impl Draw for Particle {
    fn draw(&self) {
        use Shape::*;
        match self.shape {
            Circle { radius: r } => {
                let pos = self.pos.as_vec2();
                draw_circle_lines(pos.x, pos.y, r as f32, 1.0, WHITE);
                let x = r * DMat2::from_angle(self.angle) * DVec2::X;
                let y = r * DMat2::from_angle(self.angle) * DVec2::Y;
                let pos = self.pos;
                draw_line_vec(pos + x, pos - x, 1.0, WHITE);
                draw_line_vec(pos + y, pos - y, 1.0, WHITE);
            }
            Capsule { length, radius } => {
                let capsule = geometry::Capsule::new(self.pos, self.angle, length, radius);
                let start = capsule.start;
                let end = capsule.end;
                draw_circle_lines_vec(self.pos, 5.0, 1.0, GREEN);
                draw_circle_lines_vec(start, radius, 1.0, WHITE);
                draw_circle_lines_vec(end, radius, 1.0, WHITE);
                let x = radius * DMat2::from_angle(self.angle) * DVec2::X;
                draw_line_vec(start - x, end - x, 1.0, WHITE);
                draw_line_vec(start + x, end + x, 1.0, WHITE);
            }
            HalfPlane { normal_angle } => {
                let extent = 1000.0;
                let tangent = DVec2::from_angle(normal_angle).perp();
                let from = self.pos + extent * tangent;
                let to = self.pos - extent * tangent;
                draw_line_vec(from, to, 1.0, WHITE);
            }
            _ => {
                unimplemented!("Unknown shape {:?}", self.shape)
            }
        }
    }
}

impl Draw for CollisionConstraint {
    fn draw(&self) {
        let contact = &self.contact;
        let pos_inside = contact.pos - contact.separation / 2.0 * contact.normal;
        let pos_outside = contact.pos + contact.separation / 2.0 * contact.normal;
        let normal = pos_outside + 10.0 * contact.normal;
        let color = if self.dynamic { BLUE } else { GREEN };
        draw_circle_lines_vec(pos_outside, 5.0, 2.0, color);
        draw_line_vec(pos_outside, normal, 2.0, color);
        draw_circle_lines_vec(pos_inside, 5.0, 2.0, RED);
        draw_line_vec(pos_outside, pos_inside, 2.0, RED);
    }
}

impl Draw for Engine {
    fn draw(&self) {
        for p in &self.particles {
            p.draw();
        }

        for c in &self.constraints {
            let (id_a, id_b) = c.get_ids();
            let a = &self.particles[id_a];
            let b = &self.particles[id_b];
            draw_line_vec(a.pos, b.pos, 1.0, RED);
        }

        for col in self.detect_collisions() {
            col.draw();
        }
    }
}
