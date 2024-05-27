// TODO: Think about renaming this file... shoud we still use stroke?
use glam::Vec3;
use renderer::{circle_rendering::CircleLine, colors::GREEN, transform::Transform};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_circle_line(&Transform::from_translation(&Vec3::new(0.0, 0.0, 0.0)), &CircleLine::new(100.0, GREEN, 50.0))
    }))?;
    Ok(())
}
