use glam::vec2;
use renderer::{colors::RED, stroke_rectangle::StrokeRectangle};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_stroke_rectangle(StrokeRectangle {
            pos: vec2(0., 0.),
            size: vec2(100., 300.),
            border: 25.,
            color: RED,
        })
    }))?;
    Ok(())
}
