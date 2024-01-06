use game_engine::{colors::GREEN, line_segment::LineSegment, GameEngine};
use glam::Vec2;
use winit::{event_loop::EventLoop, window::Window};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    shared::setup()?;

    let event_loop = EventLoop::new().expect("Can't create the event loop");
    let window = Window::new(&event_loop).expect("Can't create the window");
    let (mut game_engine, event_loop) = pollster::block_on(GameEngine::new(event_loop, &window))?;
    game_engine.run(event_loop, || (), &|_state, game_engine| {
        game_engine.draw_line_segment(LineSegment {
            from: Vec2::new(0., 0.),
            to: Vec2::new(200., 100.),
            color: GREEN,
        });
    })?;
    Ok(())
}
