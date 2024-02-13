use std::f64::consts::PI;

use crate::{Engine, Particle, Shape};
use glam::{dvec2, DVec2};

use super::Scenario;

const GRAVITY: DVec2 = DVec2::new(0.0, -1000.0);

fn make_circle(pos: DVec2) -> Particle {
    Particle {
        pos,
        shape: Shape::Circle { radius: 50.0 },
        ..Default::default()
    }
}

pub struct Resting {}

impl Scenario for Resting {
    fn name(&self) -> &str {
        "Resting"
    }

    fn create(&self) -> Engine {
        let mut engine = Engine {
            gravity: GRAVITY,
            ..Default::default()
        };
        engine.solver_iterations = 2;
        engine.particles = vec![
            make_circle(dvec2(-200.0, 0.0)),
            make_circle(dvec2(0.0, 0.0)),
            make_circle(dvec2(0.0, 100.0)),
            make_circle(dvec2(200.0, 0.0)),
            make_circle(dvec2(200.0, 100.0)),
            make_circle(dvec2(200.0, 200.0)),
            Particle {
                inv_mass: 0.0,
                inv_inertia: 0.0,
                pos: dvec2(0.0, -50.0),
                shape: Shape::HalfPlane {
                    normal_angle: PI / 2.0,
                },
                ..Default::default()
            },
        ];
        engine
    }
}
