use game_engine::{
    colors::{BLUE, RED},
    filled_circle::FilledCircle,
    filled_rectangle::FilledRectangle,
    Renderer,
};
use glam::Vec2;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    let (mut renderer, event_loop) = pollster::block_on(Renderer::new());
    renderer.run(event_loop, || (), &|_state, renderer| {
        renderer.draw_full_rectangle(FilledRectangle {
            pos: Vec2::new(0., 0.),
            size: Vec2::new(200., 100.),
            color: BLUE,
        });
        renderer.draw_full_circle(FilledCircle {
            pos: Vec2::new(-100., -100.),
            radius: 100.,
            color: RED,
        })
    });
}
