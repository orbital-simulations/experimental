use std::f32::consts::PI;

use game_engine::{game_engine_2_5d_parameters, GameEngine};
use glam::{vec3, Quat, Vec2};
use renderer::actor::Actor;
use renderer::colors::{GREEN, PINK, RED};
use renderer::line_rendering::Line;
use renderer::rectangle_rendering::{Rectangle, RectangleLine};
use renderer::transform::Transform;
use renderer::Renderer;
use tracing::debug;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winit::{event_loop::EventLoop, window::Window};

pub struct GameState {
    angle_1: f32,
    angle_2: f32,
    angle_3: f32,
}

#[allow(clippy::new_without_default)]
impl GameState {
    pub fn new() -> Self {
        Self {
            angle_1: 0.0,
            angle_2: 90.0,
            angle_3: 0.0,
        }
    }
}

fn setup(_game_engine: &mut GameEngine) -> GameState {
    GameState::new()
}

fn update(state: &mut GameState, _game_engine: &mut GameEngine) {
    state.angle_1 += 0.05;
    state.angle_2 += 0.10;
    state.angle_3 += 0.10;
}

fn render(state: &GameState, renderer: &mut Renderer) {
    debug!("main render");
    let angle_1 = (PI / 180.0) * state.angle_1;
    let angle_2 = (PI / 180.0) * state.angle_2;

    let mut transform = Transform::from_translation(&vec3(200.0, 0.0, 0.0));
    transform.set_rotation(&Quat::from_rotation_z(angle_2));
    let line_1 = Actor::from_line(
        transform,
        Line::new(vec3(0.0, 0.0, 1.0), vec3(60.0, 0.0, 1.0), PINK, 10.0),
    );

    let mut transform = Transform::from_translation(&vec3(200.0, 0.0, 0.0));
    transform.set_rotation(&Quat::from_rotation_z(PI / 4.0));
    let rectangle_1 = Actor::from_rectangle_line_children(
        transform,
        RectangleLine::new(Vec2::new(60.0, 60.0), RED, 5.0),
        vec![line_1],
    );

    let mut transform = Transform::from_translation(&vec3(0.0, 150.0, 0.0));
    transform.set_rotation(&Quat::from_rotation_z(PI / 3.0));
    let rectangle_2 = Actor::from_rectangle_line(
        transform,
        RectangleLine::new(Vec2::new(60.0, 60.0), RED, 5.0),
    );

    let mut transform = Transform::from_translation(&vec3(-300.0, 0.0, 0.0));
    transform.set_rotation(&Quat::from_rotation_z(PI / 5.0));
    let rectangle_3 = Actor::from_rectangle_line(
        transform,
        RectangleLine::new(Vec2::new(60.0, 60.0), RED, 5.0),
    );

    let transform = Transform::from_rotation(&Quat::from_rotation_z(angle_1));
    let pivot = Actor::invisible(transform, vec![rectangle_1, rectangle_2, rectangle_3]);

    let rectangle_transform = Transform::IDENTITY;
    let root_rectangle_actor = Actor::from_rectangle_children(
        rectangle_transform,
        Rectangle::new(Vec2::new(100.0, 150.0), GREEN),
        vec![pivot],
    );

    renderer.draw_actor(&root_rectangle_actor);
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
