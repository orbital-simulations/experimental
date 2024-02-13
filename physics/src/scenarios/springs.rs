use crate::Engine;
use glam::{dvec2, DVec2};

use super::Scenario;

pub struct Springs {}

impl Scenario for Springs {
    fn name(&self) -> &str {
        "Spring"
    }

    fn create(&self) -> Engine {
        use crate::{Particle, Shape};

        Engine {
            particles: vec![
                Particle {
                    pos: dvec2(-100.0, 50.0),
                    shape: Shape::Circle { radius: 50.0 },
                    ..Default::default()
                },
                Particle {
                    inv_mass: 0.1,
                    pos: dvec2(100.0, 0.0),
                    angle: 1.0,
                    shape: Shape::Circle { radius: 50.0 },
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }
    fn update(&self, engine: &mut Engine) {
        let p1 = &mut engine.particles[0];
        let k_linear = 50.0;
        p1.force = -k_linear * p1.pos.y * DVec2::Y;

        let p2 = &mut engine.particles[1];
        let k_angular = 20.0;
        p2.torque = -k_angular * p2.angle;
    }
}
