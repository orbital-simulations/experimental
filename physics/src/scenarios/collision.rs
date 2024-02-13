use glam::dvec2;

use crate::{Engine, Particle, Shape};

use super::Scenario;

pub struct Collision {}

impl Scenario for Collision {
    fn name(&self) -> &str {
        "Collision"
    }

    fn create(&self) -> Engine {
        let half_width = 100.0;
        Engine {
            particles: vec![
                Particle {
                    pos: dvec2(0.0 - half_width, 0.0),
                    vel: dvec2(100.0, 0.0),
                    shape: Shape::Circle { radius: 40.0 },
                    ..Default::default()
                },
                Particle {
                    inv_mass: 0.1,
                    pos: dvec2(0.0 + half_width, -30.0),
                    vel: dvec2(-50.0, 0.0),
                    shape: Shape::Circle { radius: 60.0 },
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }
}
