use glam::{Vec2, Vec3};
use renderer::{
    circle_rendering::Circle,
    colors::{BLUE, RED},
    rectangle_rendering::Rectangle,
    transform::Transform,
};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        renderer.draw_rectangle(
            &Transform::from_translation(&Vec3::new(0.0, 0.0, 0.0)).into(),
            &Rectangle::new(Vec2::new(200., 100.), BLUE),
        );
        renderer.draw_circle(
            &Transform::from_translation(&Vec3::new(-100.0, -100.0, 0.0)).into(),
            &Circle::new(100.0, RED),
        );
    }))?;
    Ok(())
}
