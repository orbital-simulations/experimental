use glam::{dvec2, vec2, DVec2};
use macroquad::{
    time::get_frame_time,
    window::{next_frame, screen_height, screen_width},
};
use physics::{Engine, Particle, Shape};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod shared;

#[derive(Default)]
pub struct GameState {
    engine: Engine,
}

const GRAVITY: DVec2 = DVec2::new(0.0, -9.81);

impl GameState {
    fn setup(&mut self) {
        self.engine.gravity = GRAVITY;
        self.engine.particles = vec![
            Particle {
                pos: dvec2(0.0, 0.0),
                shape: Shape::Circle { radius: 50.0 },
                ..Default::default()
            },
            Particle {
                inv_mass: 0.0,
                pos: dvec2(0.0, -50.0),
                shape: Shape::HalfPlane { normal_angle: 1.5 },
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
        let size = vec2(screen_width(), screen_height());
        self.engine.draw(size);
    }
}

#[macroquad::main("experimental")]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    let mut state = GameState::default();
    state.setup();

    loop {
        state.update();
        state.render();
        next_frame().await;
    }
}
