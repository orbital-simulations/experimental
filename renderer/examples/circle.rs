use glam::vec3;
use renderer::{circle_rendering::Circle, colors::GREEN, transform::Transform};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_circle(
            &Transform::from_translation(&vec3(0.0, 0.0, 0.0)),
            &Circle::new(100., GREEN),
        )
    }))?;
    Ok(())
}
