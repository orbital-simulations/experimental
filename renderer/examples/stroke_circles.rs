use glam::Vec2;
use renderer::{colors::GREEN, stroke_circle::StrokeCircle};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_stroke_circle(StrokeCircle {
            pos: Vec2::new(0., 0.),
            radius: 100.,
            border: 50.,
            color: GREEN,
        })
    }))?;
    Ok(())
}
