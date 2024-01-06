use game_engine::{
    colors::{BLUE, RED},
    filled_circle::FilledCircle,
    filled_rectangle::FilledRectangle,
    GameEngine,
};
use glam::Vec2;

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    let (event_loop, window) = shared::setup()?;
    let (mut game_engine, event_loop) = pollster::block_on(GameEngine::new(event_loop, &window))?;
    game_engine.run(event_loop, || (), &|_state, game_engine| {
        game_engine.draw_full_rectangle(FilledRectangle {
            pos: Vec2::new(0., 0.),
            size: Vec2::new(200., 100.),
            color: BLUE,
        });
        game_engine.draw_full_circle(FilledCircle {
            pos: Vec2::new(-100., -100.),
            radius: 100.,
            color: RED,
        })
    })?;
    Ok(())
}
