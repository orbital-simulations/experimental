use glam::Vec2;
use renderer::{colors::GREEN, filled_rectangle::FilledRectangle};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    let (mut render_loop, event_loop, context) = pollster::block_on(shared::Loop::setup())?;
    render_loop.run(event_loop, context, |renderer| {
        renderer.draw_full_rectangle(FilledRectangle {
            pos: Vec2::new(0., 0.),
            size: Vec2::new(200., 100.),
            color: GREEN,
        })
    })?;
    Ok(())
}
