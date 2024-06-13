use crate::{Engine, Particle, Shape};
use glam::{dvec2, DVec2};

use super::Scenario;

const GRAVITY: DVec2 = dvec2(0.0, -100.0);

pub struct InclinedFall {}

impl Scenario for InclinedFall {
    fn name(&self) -> &str {
        "Inclined Fall"
    }

    fn create(&self) -> Engine {
        Engine {
            particles: vec![
                Particle {
                    pos: dvec2(0.0, 50.0),
                    vel: dvec2(0.0, 0.0),
                    shape: Shape::Circle { radius: 50.0 },
                    ..Default::default()
                },
                Particle {
                    inv_mass: 0.0,
                    inv_inertia: 0.0,
                    pos: dvec2(0.0, -50.0),
                    shape: Shape::HalfPlane { normal_angle: 1.0 },
                    ..Default::default()
                },
            ],
            gravity: GRAVITY,
            ..Default::default()
        }
    }
}
