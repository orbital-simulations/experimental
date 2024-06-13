use std::f32::consts::PI;

use glam::{vec3, Vec2, Vec3};
use renderer::{
    circle_rendering::Circle,
    colors::{GREEN, PINK, RED, YELLOW},
    line_rendering::Line,
    rectangle_rendering::RectangleLine,
    scene_node::SceneNode,
    transform::Transform,
};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|renderer| {
        let transform = Transform::from_translation(&vec3(100.0, 0.0, 0.0));
        let line = SceneNode::from_line(
            transform,
            Line::new(vec3(0.0, 0.0, 0.0), vec3(50.0, 50.0, 0.0), PINK, 10.0),
        );

        let transform = Transform::from_translation_rotation_z(&vec3(100.0, 0.0, 0.0), PI / 4.0);
        let rectangle = SceneNode::from_rectangle_line_children(
            transform,
            RectangleLine::new(Vec2::new(60.0, 60.0), RED, 5.0),
            vec![line],
        );

        let transform = Transform::from_translation(&vec3(0.0, 100.0, 0.0));
        let circle =
            SceneNode::from_circle_children(transform, Circle::new(50.0, GREEN), vec![rectangle]);

        let transform =
            Transform::from_translation_rotation_z(&vec3(100.0, 0.0, 0.0), (PI / 180.0) * 10.0);
        let pivot = SceneNode::invisible(transform, vec![circle]);
        renderer.draw_actor(&pivot);

        renderer.draw_line(
            &Transform::IDENTITY,
            &Line::new(Vec3::ZERO, vec3(1000.0, 0.0, 0.0), YELLOW, 10.0),
        );
    }))?;
    Ok(())
}
