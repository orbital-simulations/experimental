use glam::{dvec2, DVec2};
use physics::Engine;

mod shared;

#[derive(Default)]
pub struct GameState {
    engine: Engine,
}

impl GameState {
    fn setup(&mut self) {
        use physics::{Particle, Shape};

        self.engine.particles = vec![
            Particle {
                pos: dvec2(-100.0, 50.0),
                shape: Shape::Circle { radius: 50.0 },
                ..Default::default()
            },
            Particle {
                inv_mass: 0.1,
                pos: dvec2(100.0, 0.0),
                angle: 1.0,
                shape: Shape::Circle { radius: 50.0 },
                ..Default::default()
            },
        ]
    }

    fn update(&mut self) {
        let p1 = &mut self.engine.particles[0];
        let k_linear = 50.0;
        p1.force = -k_linear * p1.pos.y * DVec2::Y;

        let p2 = &mut self.engine.particles[1];
        let k_angular = 20.0;
        p2.torque = -k_angular * p2.angle;

        let dt = macroquad::time::get_frame_time();
        self.engine.step(dt as f64);
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
