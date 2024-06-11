use glam::Vec3;
use renderer::{circle_rendering::Circle, colors::GREEN, transform::Transform};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_circle(
            &Transform::from_translation(&Vec3::new(0.0, 0.0, 0.0)).to_world(),
            &Circle::new(100., GREEN),
        )
    }))?;
    Ok(())
}
