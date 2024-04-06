use glam::Vec2;
use renderer::{
    colors::{BLUE, RED},
    filled_circle::FilledCircle,
    filled_rectangle::FilledRectangle,
};

mod shared;

fn main() -> color_eyre::eyre::Result<()> {
    pollster::block_on(shared::run(|scene| {
        scene.draw_filled_rectangle(FilledRectangle {
            pos: Vec2::new(0., 0.),
            size: Vec2::new(200., 100.),
            color: BLUE,
        });
        scene.draw_filled_circle(FilledCircle {
            pos: Vec2::new(-100., -100.),
            radius: 100.,
            color: RED,
        })
    }))?;
    Ok(())
}
