use game_engine::{colors::GREEN, filled_circle::FilledCircle, GameEngine};
use glam::Vec2;

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    let (event_loop, surface, context) = pollster::block_on(shared::setup())?;
    let renderer = renderer::Renderer::new();
    game_engine.run(event_loop, || (), &|_state, renderer| {
        renderer.draw_full_circle(FilledCircle {
            pos: Vec2::new(0., 0.),
            radius: 100.,
            color: GREEN,
        })
    })?;
    Ok(())
}
