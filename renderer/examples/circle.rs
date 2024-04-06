use glam::Vec2;
use renderer::{colors::GREEN, filled_circle::FilledCircle};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|scene| {
        scene.draw_filled_circle(FilledCircle {
            pos: Vec2::new(0., 0.),
            radius: 100.,
            color: GREEN,
        })
    }))?;
    Ok(())
}
