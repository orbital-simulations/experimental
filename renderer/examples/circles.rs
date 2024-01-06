use game_engine::{colors::GREEN, filled_circle::FilledCircle, GameEngine};
use glam::Vec2;

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    let (event_loop, window) = shared::setup()?;
    let (mut game_engine, event_loop) = pollster::block_on(GameEngine::new(event_loop, &window))?;
    game_engine.run(event_loop, || (), &|_state, renderer| {
        renderer.draw_full_circle(FilledCircle {
            pos: Vec2::new(0., 0.),
            radius: 100.,
            color: GREEN,
        })
    })?;
    Ok(())
}
