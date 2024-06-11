use std::{f64::consts::PI, iter::repeat_with};

use game_engine::{game_engine_2_5d_parameters, GameEngine};
use glam::{dvec2, vec3, DVec2};
use physics::{Engine, Particle, Shape};
use rand::Rng;
use renderer::line_rendering::Line;
use renderer::Renderer;
use renderer::{
    circle_rendering::Circle,
    colors::{RED, YELLOW},
    transform::Transform,
};
use tracing::debug;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winit::{event_loop::EventLoop, window::Window};

const CIRCLE_NUMBER: usize = 100;

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

fn setup(_game_engine: &mut GameEngine) -> GameState {
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
    game_state.engine.particles.push(Particle {
        inv_mass: 0.0,
        inv_inertia: 0.0,
        pos: dvec2(0.0, 500.0),
        shape: Shape::HalfPlane {
            normal_angle: -PI / 2.,
        },
        ..Default::default()
    });
    game_state.engine.particles.push(Particle {
        inv_mass: 0.0,
        inv_inertia: 0.0,
        pos: dvec2(0.0, -500.0),
        shape: Shape::HalfPlane {
            normal_angle: PI / 2.,
        },
        ..Default::default()
    });
    game_state.engine.particles.push(Particle {
        inv_mass: 0.0,
        inv_inertia: 0.0,
        pos: dvec2(500.0, 0.0),
        shape: Shape::HalfPlane { normal_angle: -PI },
        ..Default::default()
    });
    game_state.engine.particles.push(Particle {
        inv_mass: 0.0,
        inv_inertia: 0.0,
        pos: dvec2(-500.0, 0.0),
        shape: Shape::HalfPlane { normal_angle: 0. },
        ..Default::default()
    });

    game_state
}

fn update(state: &mut GameState, game_engine: &mut GameEngine) {
    let dt = game_engine.last_frame_delta;

    state.engine.step(dt as f64);
}

fn render(state: &GameState, renderer: &mut Renderer) {
    debug!("main render");
    for p in &state.engine.particles {
        match p.shape {
            Shape::Circle { radius } => {
                renderer.draw_circle(
                    &Transform::from_translation(&vec3(
                        p.pos.as_vec2().x,
                        p.pos.as_vec2().y,
                        0.0,
                    ))
                    .into(),
                    &Circle::new(radius as f32, RED),
                );
            }
            Shape::HalfPlane { normal_angle } => {
                let extent = 10000.0;
                let tangent = DVec2::from_angle(normal_angle).perp();
                let from: DVec2 = p.pos + extent * tangent;
                let to: DVec2 = p.pos - extent * tangent;
                renderer.draw_line(
                    &Transform::IDENTITY.into(),
                    &Line {
                        from: vec3(from.x as f32, from.y as f32, 0.0),
                        to: vec3(to.x as f32, to.y as f32, 0.0),
                        color: YELLOW,
                        width: 3.,
                    },
                );
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
    let event_loop = EventLoop::new()?;
    let window = Window::new(&event_loop)?;
    let (mut game_engine, event_loop) = pollster::block_on(GameEngine::new(
        event_loop,
        &window,
        game_engine_2_5d_parameters(),
    ))?;
    game_engine.run(event_loop, setup, &update, &render)?;
    Ok(())
}
