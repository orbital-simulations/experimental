use glam::Vec2;
use renderer::{colors::GREEN, line_segment::LineSegment};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_line_segment(LineSegment {
            from: Vec2::new(0., 0.),
            to: Vec2::new(200., 100.),
            color: GREEN,
            width: 1.,
        });
    }))?;
    Ok(())
}
