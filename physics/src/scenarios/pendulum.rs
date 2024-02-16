use crate::{
    constraint::{ConstraintEnum, DistanceConstraint},
    Engine,
};
use glam::{dvec2, DVec2};

use super::Scenario;

const GRAVITY: DVec2 = dvec2(0.0, -1000.0);

pub struct Pendulum {}

impl Scenario for Pendulum {
    fn name(&self) -> &str {
        "Pendulum"
    }

    fn create(&self) -> Engine {
        use crate::{Particle, Shape};

        let mut engine = Engine {
            gravity: GRAVITY,
            ..Default::default()
        };
        engine.particles = vec![
            Particle {
                inv_mass: 0.0,
                inv_inertia: 0.0,
                pos: dvec2(0.0, 100.0),
                shape: Shape::Circle { radius: 10.0 },
                ..Default::default()
            },
            Particle {
                pos: dvec2(100.0, 100.0),
                vel: dvec2(0.0, 0.0),
                shape: Shape::Circle { radius: 20.0 },
                ..Default::default()
            },
            Particle {
                pos: dvec2(200.0, 100.0),
                vel: dvec2(0.0, 0.0),
                shape: Shape::Circle { radius: 20.0 },
                ..Default::default()
            },
        ];

        engine.constraints = vec![
            ConstraintEnum::Distance(DistanceConstraint {
                id_a: 0,
                id_b: 1,
                distance: 100.0,
            }),
            ConstraintEnum::Distance(DistanceConstraint {
                id_a: 1,
                id_b: 2,
                distance: 100.0,
            }),
        ];
        engine
    }
}
