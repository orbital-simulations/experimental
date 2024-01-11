use std::f64::consts::PI;

use glam::{dvec2, DVec2};
use macroquad::{time::get_frame_time, window::next_frame};
use physics::{Engine, Particle, Shape};

mod shared;

#[derive(Default)]
pub struct GameState {
    engine: Engine,
}

const GRAVITY: DVec2 = DVec2::new(0.0, -100.0);

impl GameState {
    fn setup(&mut self) {
        self.engine.gravity = GRAVITY;
        let radius = 50.0;
        let length = 100.0;
        self.engine.particles = vec![
            Particle {
                pos: dvec2(0.0, 100.0),
                vel: dvec2(0.0, 0.0),
                inv_inertia: 1.0 / 1000.0,
                angle: PI / 2.0 + 0.1,
                shape: Shape::Capsule { length, radius },
                ..Default::default()
            },
            Particle {
                inv_mass: 0.0,
                inv_inertia: 0.0,
                pos: dvec2(0.0, 0.0),
                shape: Shape::HalfPlane {
                    normal_angle: PI / 2.0,
                },
                ..Default::default()
            },
        ]
    }

    fn update(&mut self) {
        let dt = get_frame_time() as f64;
        self.engine.step(dt);
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

    for _i in 1.. {
        state.update();
        state.render();
        next_frame().await;
    }
}
