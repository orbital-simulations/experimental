use glam::dvec2;
use physics::Engine;

#[derive(Default)]
pub struct GameState {
    engine: Engine,
}

impl GameState {
    fn setup(&mut self) {
        use physics::Particle;
        use physics::Shape;

        let half_width = 50.0;
        let speed = 10.0;
        self.engine.particles = vec![
            Particle {
                mass: 1.0,
                pos: dvec2(400.0 - half_width, 300.0),
                vel: dvec2(speed, 0.0),
                shape: Shape::Circle(40.0),
            },
            Particle {
                mass: 1.0,
                pos: dvec2(400.0 + half_width, 330.0),
                vel: dvec2(-speed, 0.0),
                shape: Shape::Circle(60.0),
            },
        ]
    }

    fn update(&mut self) {
        use macroquad::time::get_frame_time;

        let dt = get_frame_time();
        self.engine.step(dt as f64);
    }

    fn render(&self) {
        use macroquad::{color::WHITE, shapes::draw_circle_lines};
        use physics::Shape;

        for p in &self.engine.particles {
            let pos = p.pos.as_vec2();
            if let Shape::Circle(r) = p.shape {
                draw_circle_lines(pos.x, pos.y, r as f32, 1.0, WHITE);
            }
        }

        for col in self.engine.detect_collisions() {
            use macroquad::color::RED;
            use macroquad::shapes::draw_line;

            let contact = col.contact;
            let pos_inside = (contact.pos + contact.separation * contact.normal).as_vec2();
            let pos = contact.pos.as_vec2();
            draw_line(pos.x, pos.y, pos_inside.x, pos_inside.y, 2.0, RED);
        }
    }
}

#[macroquad::main("experimental")]
async fn main() {
    use macroquad::window::next_frame;

    let mut state = GameState::default();
    state.setup();

    loop {
        state.update();
        state.render();
        next_frame().await;
    }
}
