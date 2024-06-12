use glam::{vec2, vec3};
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
            &Transform::from_translation(&vec3(0.0, 0.0, 0.0)),
            &Rectangle::new(vec2(200., 100.), BLUE),
        );
        renderer.draw_circle(
            &Transform::from_translation(&vec3(-100.0, -100.0, 0.0)),
            &Circle::new(100.0, RED),
        );
    }))?;
    Ok(())
}
