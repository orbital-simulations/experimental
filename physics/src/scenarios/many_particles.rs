use std::f64::consts::PI;

use glam::{dvec2, DVec2};
use rand::Rng as _;

use crate::{Engine, Particle, Shape};

use super::Scenario;

const CIRCLE_NUMBER: usize = 100;
const GRAVITY: DVec2 = dvec2(0.0, -9.81);

pub struct ManyParticles {}

impl Scenario for ManyParticles {
    fn name(&self) -> &str {
        "Many Particles"
    }

    fn create(&self) -> Engine {
        let mut engine = Engine {
            gravity: GRAVITY,
            ..Default::default()
        };

        let mut rng = rand::thread_rng();
        let pos_limit = 500.0;
        let vel_limit = 50.0;
        engine.particles.extend(
            std::iter::repeat_with(|| Particle {
                inv_mass: rng.gen_range(1.0..3.0),
                pos: dvec2(
                    rng.gen_range(-pos_limit..pos_limit),
                    rng.gen_range(-pos_limit..pos_limit),
                ),
                vel: dvec2(
                    rng.gen_range(-vel_limit..vel_limit),
                    rng.gen_range(-vel_limit..vel_limit),
                ),
                shape: Shape::Circle { radius: 10. },
                ..Default::default()
            })
            .take(CIRCLE_NUMBER),
        );
        engine.particles.push(Particle {
            inv_mass: 0.0,
            inv_inertia: 0.0,
            pos: dvec2(0.0, 500.0),
            shape: Shape::HalfPlane {
                normal_angle: -PI / 2.,
            },
            ..Default::default()
        });
        engine.particles.push(Particle {
            inv_mass: 0.0,
            inv_inertia: 0.0,
            pos: dvec2(0.0, -500.0),
            shape: Shape::HalfPlane {
                normal_angle: PI / 2.,
            },
            ..Default::default()
        });
        engine.particles.push(Particle {
            inv_mass: 0.0,
            inv_inertia: 0.0,
            pos: dvec2(500.0, 0.0),
            shape: Shape::HalfPlane { normal_angle: -PI },
            ..Default::default()
        });
        engine.particles.push(Particle {
            inv_mass: 0.0,
            inv_inertia: 0.0,
            pos: dvec2(-500.0, 0.0),
            shape: Shape::HalfPlane { normal_angle: 0. },
            ..Default::default()
        });

        engine
    }
}
