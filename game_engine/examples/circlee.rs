use std::{f64::consts::PI, iter::repeat_with};

use game_engine::{game_engine_2_5d_parameters, GameEngine};
use glam::{dvec2, DVec2, Vec2, Vec3};
use physics::{Engine, Particle, Shape};
use rand::Rng;
use renderer::{
    circle_rendering::{Circle, CircleLine}, colors::{RED, YELLOW}, rectangle_rendering::{Rectangle, RectangleLine}, transform::Transform
};
use renderer::renderer_api::Renderer;
use tracing::debug;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winit::{event_loop::EventLoop, window::Window};

const CIRCLE_NUMBER: usize = 1000;

pub struct GameState {
}

const GRAVITY: DVec2 = DVec2::new(0.0, -9.81);

fn setup(_game_engine: &mut GameEngine) -> GameState {
    GameState{}
}

fn update(state: &mut GameState, game_engine: &mut GameEngine) {
}

fn render(state: &GameState, renderer: &mut Renderer) {
    debug!("main render");
//    renderer.draw_circle(&Transform::from_translation(&Vec3::new(0.0, 0.0, 0.0)), &Circle::new(500., RED));
//    renderer.draw_circle_line(&Transform::from_translation(&Vec3::new(0.0, 0.0, 0.0)), &CircleLine::new(10., RED, 5.));
//    renderer.draw_rectangle(&Transform::from_translation(&Vec3::new(0.0, 0.0, 0.0)), &Rectangle::new(Vec2::new(10.0, 50.0), RED));
    renderer.draw_rectangle_line(&Transform::from_translation(&Vec3::new(0.0, 0.0, 0.0)), &RectangleLine::new(Vec2::new(20.0, 100.0), RED, 5.0));
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
