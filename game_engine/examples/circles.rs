use game_engine::{filled_circle::FilledCircle, Renderer};
use glam::{Vec2, Vec3};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    let (mut renderer, event_loop) = pollster::block_on(Renderer::new());
    renderer.run(event_loop, || (), &|_state, renderer| {
        renderer.draw_full_circle(FilledCircle {
            pos: Vec2::new(0., 0.),
            radius: 100.,
            color: Vec3::new(0., 1., 0.),
        })
    });
}
