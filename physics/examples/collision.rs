use glam::{dvec2, vec2};
use macroquad::window::{screen_height, screen_width};
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
                pos: dvec2(0.0 - half_width, 0.0),
                vel: dvec2(100.0, 0.0),
                shape: Shape::Circle { radius: 40.0 },
                ..Default::default()
            },
            Particle {
                inv_mass: 0.1,
                pos: dvec2(0.0 + half_width, -30.0),
                vel: dvec2(-50.0, 0.0),
                shape: Shape::Circle { radius: 60.0 },
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
        let size = vec2(screen_width(), screen_height());
        self.engine.draw(size);
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
