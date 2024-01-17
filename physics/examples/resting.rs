use std::f64::consts::PI;

use glam::{dvec2, DVec2};
use macroquad::{time::get_frame_time, window::next_frame};
use physics::{Engine, Particle, Shape};

mod shared;

#[derive(Default)]
pub struct GameState {
    engine: Engine,
}

const GRAVITY: DVec2 = DVec2::new(0.0, -10.0);
const GROUND: f64 = -150.0;

fn make_circle(pos: DVec2) -> Particle {
    let radius = 50.0;
    Particle {
        pos: pos + dvec2(0.0, GROUND + radius),
        shape: Shape::Circle { radius },
        ..Default::default()
    }
}

fn make_capsule(pos: DVec2, angle: f64) -> Particle {
    let radius = 20.0;
    Particle {
        pos: pos + dvec2(0.0, GROUND + radius),
        angle,
        shape: Shape::Capsule {
            radius,
            length: 50.0,
        },
        ..Default::default()
    }
}

impl GameState {
    fn setup(&mut self) {
        self.engine.gravity = GRAVITY;
        self.engine.particles = vec![
            make_capsule(dvec2(-300.0, 0.0), PI / 2.0),
            make_capsule(dvec2(-150.0, 0.0), PI / 2.0),
            make_circle(dvec2(-150.0, 40.0)),
            make_circle(dvec2(0.0, 0.0)),
            make_circle(dvec2(150.0, 0.0)),
            make_circle(dvec2(150.0, 100.0)),
            make_circle(dvec2(300.0, 0.0)),
            make_circle(dvec2(300.0, 100.0)),
            make_circle(dvec2(300.0, 200.0)),
            Particle {
                inv_mass: 0.0,
                inv_inertia: 0.0,
                pos: dvec2(0.0, GROUND),
                shape: Shape::HalfPlane {
                    normal_angle: PI / 2.0,
                },
                ..Default::default()
            },
        ]
    }

    fn update(&mut self) {
        let dt = get_frame_time() as f64;
        //let dt = 1.0 / 120.0;
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

    loop {
        state.update();
        state.render();
        next_frame().await;
    }
}
