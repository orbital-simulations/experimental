use glam::Vec3;
use renderer::{colors::GREEN, line_rendering::Line, transform::Transform};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_line(
            &Transform::IDENTITY.into(),
            &Line {
                from: Vec3::new(0.0, 0.0, 0.0),
                to: Vec3::new(200.0, 100.0, 0.0),
                color: GREEN,
                width: 10.,
            },
        );
    }))?;
    Ok(())
}
