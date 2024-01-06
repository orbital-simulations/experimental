use game_engine::{
    colors::{BLUE, RED},
    filled_circle::FilledCircle,
    filled_rectangle::FilledRectangle,
    GameEngine,
};
use glam::Vec2;
use winit::{event_loop::EventLoop, window::Window};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    shared::setup()?;

    let event_loop = EventLoop::new().expect("Can't create the event loop");
    let window = Window::new(&event_loop).expect("Can't create the window");
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
