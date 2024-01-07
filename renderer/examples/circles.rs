use glam::Vec2;
use renderer::{colors::GREEN, filled_circle::FilledCircle};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    let (mut render_loop, event_loop) = pollster::block_on(shared::Loop::setup())?;
    render_loop.run(event_loop, |renderer| {
        renderer.draw_full_circle(FilledCircle {
            pos: Vec2::new(0., 0.),
            radius: 100.,
            color: GREEN,
        })
    })?;
    Ok(())
}
