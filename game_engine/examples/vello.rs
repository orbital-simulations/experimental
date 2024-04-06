use game_engine::{game_engine_2_5d_parameters, GameEngine, RenderingBackend, Scene};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winit::{event_loop::EventLoop, window::Window};

#[derive(Default)]
pub struct GameState {}

fn setup(_game_engine: &mut GameEngine) -> GameState {
    GameState::default()
}

fn update(_state: &mut GameState, _game_engine: &mut GameEngine) {}

fn render(_state: &GameState, scene: &mut Scene) {
    let Scene::Vello(scene) = scene else {
        panic!("Expected a vello scene");
    };
    scene.fill(
        vello::peniko::Fill::NonZero,
        Default::default(),
        vello::peniko::Color::rgb8(242, 140, 168),
        None,
        &vello::kurbo::Circle::new((420.0, 200.0), 120.0),
    );

    /*
    // Draw more stuff
        scene.push_layer(...);
        scene.fill(...);
        scene.stroke(...);
        scene.pop_layer(...);
         */
}

fn main() -> eyre::Result<()> {
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
        game_engine_2_5d_parameters(RenderingBackend::Vello),
    ))?;
    game_engine.run(event_loop, setup, &update, &render)?;
    Ok(())
}
