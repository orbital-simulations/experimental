use std::iter::repeat_with;

use game_engine::{colors::RED, filled_circle::FilledCircle, Renderer};
use glam::{dvec2, DVec2};
use physics::{Engine, Particle, Shape};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const CIRCLE_NUMBER: usize = 1000;

pub struct GameState {
    engine: Engine,
}

#[allow(clippy::new_without_default)]
impl GameState {
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
        }
    }
}

const GRAVITY: DVec2 = DVec2::new(0.0, -9.81);

fn setup() -> GameState {
    let mut game_state = GameState::new();
    game_state.engine.gravity = GRAVITY;
    game_state.engine.particles.extend(
        repeat_with(|| Particle {
            mass: (rand::random::<f64>() * 2.) + 1.,
            vel: DVec2::ZERO,
            shape: Shape::Circle(10.),
            pos: dvec2(
                (rand::random::<f64>() * 1000.) - 500.,
                (rand::random::<f64>() * 1000.) - 500.,
            ),
            ..Default::default()
        })
        .take(CIRCLE_NUMBER),
    );
    game_state
}


fn update(state: &mut GameState, renderer: &mut Renderer) {
    let dt = renderer.last_frame_delta;

    state.engine.step(dt as f64);

    for p in &state.engine.particles {
        if let Shape::Circle(radius) = p.shape {
            renderer.draw_full_circle(FilledCircle::new(p.pos.as_vec2(),radius as f32, RED));
        }
        renderer.draw_full_circle(FilledCircle::new(p.pos.as_vec2(), 10., RED));
    }
}

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    let (mut renderer, event_loop) = pollster::block_on(Renderer::new());
    renderer.run(event_loop, setup, &update);
}
