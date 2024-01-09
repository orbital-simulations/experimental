use glam::{dvec2, DVec2};
use physics::{
    constraint::{ConstraintEnum, DistanceConstraint},
    Engine,
};

mod shared;

#[derive(Default)]
pub struct GameState {
    engine: Engine,
}

const GRAVITY: DVec2 = dvec2(0.0, -1000.0);

impl GameState {
    fn setup(&mut self) {
        use physics::{Particle, Shape};

        self.engine.gravity = GRAVITY;
        self.engine.particles = vec![
            Particle {
                inv_mass: 0.0,
                inv_inertia: 0.0,
                pos: dvec2(0.0, 100.0),
                shape: Shape::Circle { radius: 10.0 },
                ..Default::default()
            },
            Particle {
                pos: dvec2(100.0, 100.0),
                vel: dvec2(0.0, 0.0),
                shape: Shape::Circle { radius: 20.0 },
                ..Default::default()
            },
            Particle {
                pos: dvec2(200.0, 100.0),
                vel: dvec2(0.0, 0.0),
                shape: Shape::Circle { radius: 20.0 },
                ..Default::default()
            },
        ];

        self.engine.constraints = vec![
            ConstraintEnum::Distance(DistanceConstraint {
                id_a: 0,
                id_b: 1,
                distance: 100.0,
            }),
            ConstraintEnum::Distance(DistanceConstraint {
                id_a: 1,
                id_b: 2,
                distance: 100.0,
            }),
        ];
    }

    fn update(&mut self) {
        let dt = macroquad::time::get_frame_time() as f64;
        self.engine.step(dt);
    }

    fn render(&self) {
        use shared::draw::Draw;
        self.engine.draw();
    }
}

#[macroquad::main("experimental")]
async fn main() {
    use macroquad::window::next_frame;
    shared::setup();
    let mut state = GameState::default();
    state.setup();

    loop {
        state.update();
        state.render();
        next_frame().await;
    }
}
