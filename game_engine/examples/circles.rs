use game_engine::{filled_circle::FilledCircle, GameEngine};
use glam::{Vec2, Vec3};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    shared::setup()?;

    let (mut game_engine, event_loop) = pollster::block_on(GameEngine::new())?;
    game_engine.run(event_loop, || (), &|_state, renderer| {
        renderer.draw_full_circle(FilledCircle {
            pos: Vec2::new(0., 0.),
            radius: 100.,
            color: Vec3::new(0., 1., 0.),
        })
    })?;
    Ok(())
}
