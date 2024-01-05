use glam::DVec2;
use macroquad::color::Color;

use physics::{Collision, Particle, Shape};

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
            _ => {
                unimplemented!("Uknown shape {:?}", self.shape)
            }
        }
    }
}

impl Draw for Collision {
    fn draw(&self) {
        use macroquad::color::RED;
        let contact = &self.contact;
        let pos_inside = contact.pos + contact.separation * contact.normal;
        draw_vec_line(contact.pos, pos_inside, 2.0, RED);
    }
}
