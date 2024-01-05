use std::iter::repeat_with;

use game_engine::{colors::RED, filled_circle::FilledCircle, GameEngine};
use glam::{dvec2, DVec2};
use physics::{Engine, Particle, Shape};
use rand::Rng;
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

    let mut rng = rand::thread_rng();
    let pos_limit = 500.0;
    let vel_limit = 50.0;
    game_state.engine.particles.extend(
        repeat_with(|| Particle {
            inv_mass: rng.gen_range(1.0..3.0),
            pos: dvec2(
                rng.gen_range(-pos_limit..pos_limit),
                rng.gen_range(-pos_limit..pos_limit),
            ),
            vel: dvec2(
                rng.gen_range(-vel_limit..vel_limit),
                rng.gen_range(-vel_limit..vel_limit),
            ),
            shape: Shape::Circle { radius: 10. },
            ..Default::default()
        })
        .take(CIRCLE_NUMBER),
    );

    game_state
}

fn update(state: &mut GameState, game_engine: &mut GameEngine) {
    let dt = game_engine.last_frame_delta;

    state.engine.step(dt as f64);

    for p in &state.engine.particles {
        match p.shape {
            Shape::Circle { radius } => {
                game_engine.draw_full_circle(FilledCircle::new(
                    p.pos.as_vec2(),
                    radius as f32,
                    RED,
                ));
            }
            Shape::HalfPlane { .. } => {
                unimplemented!("Render a half-plane")
            }
            _ => {
                unimplemented!("Render unknown shape {:?}", p.shape)
            }
        }
    }
}

fn main() -> color_eyre::eyre::Result<()> {
    let fmt_layer = fmt::layer().pretty();
    let filter_layer = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();
    color_eyre::install()?;
    let (mut game_engine, event_loop) = pollster::block_on(GameEngine::new())?;
    game_engine.run(event_loop, setup, &update)?;
    Ok(())
}
