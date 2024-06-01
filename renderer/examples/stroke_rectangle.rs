// TODO: Think about renaming this file... shoud we still use stroke?
use glam::{vec2, Vec3};
use renderer::{colors::RED, rectangle_rendering::RectangleLine, transform::Transform};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_rectangle_line(
            &Transform::from_translation(&Vec3::new(0.0, 0.0, 0.0)),
            &RectangleLine::new(vec2(100.0, 300.0), RED, 25.0),
        );
    }))?;
    Ok(())
}
