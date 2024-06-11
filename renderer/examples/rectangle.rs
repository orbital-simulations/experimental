use glam::{vec2, vec3};
use renderer::{colors::GREEN, rectangle_rendering::Rectangle, transform::Transform};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_rectangle(
            &Transform::from_translation(&vec3(0.0, 0.0, 0.0)).into(),
            &Rectangle::new(vec2(200., 100.), GREEN),
        );
    }))?;
    Ok(())
}
