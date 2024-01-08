use glam::Vec2;
use renderer::{colors::GREEN, filled_rectangle::FilledRectangle};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_full_rectangle(FilledRectangle {
            pos: Vec2::new(0., 0.),
            size: Vec2::new(200., 100.),
            color: GREEN,
        })
    }))?;
    Ok(())
}
