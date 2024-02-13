use glam::{dvec2, DVec2};

use crate::{Engine, Particle, Shape};

use super::Scenario;

const GRAVITY: DVec2 = dvec2(0.0, -9.81);

pub struct SimpleFall {}

impl Scenario for SimpleFall {
    fn name(&self) -> &str {
        "Simple Fall"
    }

    fn create(&self) -> Engine {
        let half_width = 100.0;
        Engine {
            particles: vec![Particle {
                pos: dvec2(0.0 - half_width, 0.0),
                shape: Shape::Circle { radius: 40.0 },
                ..Default::default()
            }],
            gravity: GRAVITY,
            ..Default::default()
        }
    }
}
