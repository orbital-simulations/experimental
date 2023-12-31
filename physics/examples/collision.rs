use glam::dvec2;
use physics::Engine;

mod shared;

#[derive(Default)]
pub struct GameState {
    engine: Engine,
}

impl GameState {
    fn setup(&mut self) {
        use physics::{Particle, Shape};

        let half_width = 100.0;
        self.engine.particles = vec![
            Particle {
                pos: dvec2(400.0 - half_width, 300.0),
                vel: dvec2(100.0, 0.0),
                shape: Shape::Circle(40.0),
                ..Default::default()
            },
            Particle {
                mass: 10.0,
                pos: dvec2(400.0 + half_width, 330.0),
                vel: dvec2(-50.0, 0.0),
                shape: Shape::Circle(60.0),
                ..Default::default()
            },
        ]
    }

    fn update(&mut self) {
        use macroquad::time::get_frame_time;

        let dt = get_frame_time();
        self.engine.step(dt as f64);
    }

    fn render(&self) {
        use shared::draw::Draw;

        for p in &self.engine.particles {
            p.draw();
        }

        for col in self.engine.detect_collisions() {
            col.draw();
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
