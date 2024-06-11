use glam::{Vec2, Vec3};
use renderer::{colors::GREEN, rectangle_rendering::Rectangle, transform::Transform};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_rectangle(
            &Transform::from_translation(&Vec3::new(0.0, 0.0, 0.0)).to_world(),
            &Rectangle::new(Vec2::new(200., 100.), GREEN),
        );
    }))?;
    Ok(())
}
