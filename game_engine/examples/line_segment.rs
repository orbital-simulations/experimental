use game_engine::{line_segment::LineSegment, GameEngine};
use glam::{Vec2, Vec3};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    shared::setup()?;

    let (mut game_engine, event_loop) = pollster::block_on(GameEngine::new())?;
    game_engine.run(event_loop, || (), &|_state, game_engine| {
        game_engine.draw_line_segment(LineSegment {
            from: Vec2::new(0., 0.),
            to: Vec2::new(200., 100.),
            color: Vec3::new(0., 1., 0.),
        });
    })?;
    Ok(())
}
