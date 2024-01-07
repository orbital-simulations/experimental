use glam::Vec2;
use renderer::{
    colors::{BLUE, RED},
    filled_circle::FilledCircle,
    filled_rectangle::FilledRectangle,
};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    let (mut render_loop, event_loop) = pollster::block_on(shared::Loop::setup())?;
    render_loop.run(event_loop, |renderer| {
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
    })?;
    Ok(())
}
