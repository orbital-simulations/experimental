use std::f64::consts::PI;

use glam::{dvec2, DVec2};
use macroquad::{time::get_frame_time, window::next_frame};
use physics::{Engine, Particle, Shape};

mod shared;

#[derive(Default)]
pub struct GameState {
    engine: Engine,
}

const GRAVITY: DVec2 = DVec2::new(0.0, -1000.0);

fn make_circle(pos: DVec2) -> Particle {
    Particle {
        pos,
        shape: Shape::Circle { radius: 50.0 },
        ..Default::default()
    }
}

impl GameState {
    fn setup(&mut self) {
        self.engine.gravity = GRAVITY;
        self.engine.solver_iterations = 2;
        self.engine.particles = vec![
            make_circle(dvec2(-200.0, 0.0)),
            make_circle(dvec2(0.0, 0.0)),
            make_circle(dvec2(0.0, 100.0)),
            make_circle(dvec2(200.0, 0.0)),
            make_circle(dvec2(200.0, 100.0)),
            make_circle(dvec2(200.0, 200.0)),
            Particle {
                inv_mass: 0.0,
                inv_inertia: 0.0,
                pos: dvec2(0.0, -50.0),
                shape: Shape::HalfPlane {
                    normal_angle: PI / 2.0,
                },
                ..Default::default()
            },
        ]
    }

    fn update(&mut self) {
        let dt = get_frame_time();
        self.engine.step(dt as f64);
    }

    fn render(&self) {
        use shared::draw::Draw;
        self.engine.draw();
    }
}

#[macroquad::main("experimental")]
async fn main() {
    shared::setup();
    let mut state = GameState::default();
    state.setup();

    loop {
        state.update();
        state.render();
        next_frame().await;
    }
}
