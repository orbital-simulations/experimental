use glam::DVec2;
use macroquad::color::{Color, RED};

use physics::{
    constraint::{CollisionConstraint, Constraint},
    Engine, Particle, Shape,
};

pub fn draw_vec_line(from: DVec2, to: DVec2, thickness: f32, color: Color) {
    use macroquad::shapes::draw_line;
    let from = from.as_vec2();
    let to = to.as_vec2();
    draw_line(from.x, from.y, to.x, to.y, thickness, color)
}

pub trait Draw {
    fn draw(&self);
}

impl Draw for Particle {
    fn draw(&self) {
        use glam::DMat2;
        use macroquad::color::WHITE;
        use macroquad::shapes::draw_circle_lines;
        use Shape::*;
        match self.shape {
            Circle { radius: r } => {
                let pos = self.pos.as_vec2();
                draw_circle_lines(pos.x, pos.y, r as f32, 1.0, WHITE);
                let x = r * DMat2::from_angle(self.angle) * DVec2::X;
                let y = r * DMat2::from_angle(self.angle) * DVec2::Y;
                let pos = self.pos;
                draw_vec_line(pos + x, pos - x, 1.0, WHITE);
                draw_vec_line(pos + y, pos - y, 1.0, WHITE);
            }
            HalfPlane { normal_angle } => {
                let extent = 1000.0;
                let tangent = DVec2::from_angle(normal_angle).perp();
                let from = self.pos + extent * tangent;
                let to = self.pos - extent * tangent;
                draw_vec_line(from, to, 1.0, WHITE);
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
        let pos_inside = contact.pos + contact.separation * contact.normal;
        draw_vec_line(contact.pos, pos_inside, 2.0, RED);
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
            draw_vec_line(a.pos, b.pos, 1.0, RED);
        }

        for col in self.detect_collisions() {
            col.draw();
        }
    }
}
