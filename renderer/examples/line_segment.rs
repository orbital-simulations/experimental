use glam::vec3;
use renderer::{colors::GREEN, line_rendering::Line, transform::Transform};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_line(
            &Transform::IDENTITY,
            &Line {
                from: vec3(0.0, 0.0, 0.0),
                to: vec3(200.0, 100.0, 0.0),
                color: GREEN,
                width: 10.,
            },
        );
    }))?;
    Ok(())
}
