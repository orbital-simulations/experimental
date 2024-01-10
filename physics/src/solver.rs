use glam::{dvec3, DMat3, DVec3};
use tracing::{instrument, trace, trace_span, warn};

use crate::{
    constraint::{Constraint, ConstraintEnum},
    Particle,
};

pub trait Solver {
    fn solve(&self, particles: &mut [Particle], constraints: &mut [ConstraintData]);
}

// Some variables do not change during solving,
// so we precompute them at the beginning and store them.
#[derive(Clone, Debug)]
pub struct ConstraintData<'a> {
    jacobian: (DVec3, DVec3),
    target_velocity: f64,
    total_impulse: f64,
    constraint: &'a ConstraintEnum,
}

impl<'a> ConstraintData<'a> {
    pub fn from_constraint(
        c: &'a ConstraintEnum,
        particles: &[Particle],
        dt: f64,
    ) -> ConstraintData<'a> {
        let (id_a, id_b) = c.get_ids();
        let a = &particles[id_a];
        let b = &particles[id_b];
        ConstraintData {
            jacobian: c.jacobian(a, b),
            target_velocity: c.target_velocity(a, b, dt),
            constraint: c,
            total_impulse: 0.0,
        }
    }

    fn relative_velocity(&self, a: &Particle, b: &Particle) -> f64 {
        let (j1, j2) = self.jacobian;
        let v1 = dvec3(a.vel.x, a.vel.y, a.omega);
        let v2 = dvec3(b.vel.x, b.vel.y, b.omega);
        let v_rel = j1.dot(v1) + j2.dot(v2);
        trace!("Velocity 1: {v1}, velocity 2: {v2}, relative velocity: {v_rel}");
        v_rel
    }
}

#[derive(Clone, Debug)]
pub struct SequentialImpulseSolver {
    pub dt: f64,
    pub iterations: usize,
}

// TODO: document solver
// see https://github.com/orbital-simulations/experimental/issues/50
impl SequentialImpulseSolver {
    fn find_impulse(&self, a: &Particle, b: &Particle, c: &mut ConstraintData<'_>) -> f64 {
        // TODO: matrices should be precomputed
        // see https://github.com/orbital-simulations/experimental/issues/52
        let m1_inv = DMat3::from_diagonal(dvec3(a.inv_mass, a.inv_mass, a.inv_inertia));
        let m2_inv = DMat3::from_diagonal(dvec3(b.inv_mass, b.inv_mass, b.inv_inertia));
        let v_rel = c.relative_velocity(a, b);
        let v_target = c.target_velocity;
        let (j1, j2) = c.jacobian;
        let new_lambda = (v_target - v_rel) / (j1.dot(m1_inv * j1) + j2.dot(m2_inv * j2));
        // TODO should the clamping be done only for the inequality constraints?
        let new_total = (c.total_impulse + new_lambda).max(0.0);
        let lambda = new_total - c.total_impulse;
        c.total_impulse += lambda;
        trace!("Impulse magnitude: {lambda}");
        lambda
    }

    fn apply(&self, a: &mut Particle, b: &mut Particle, c: &ConstraintData<'_>, impulse: f64) {
        // TODO: matrices should be precomputed
        // see https://github.com/orbital-simulations/experimental/issues/52
        let m1_inv = DMat3::from_diagonal(dvec3(a.inv_mass, a.inv_mass, a.inv_inertia));
        let m2_inv = DMat3::from_diagonal(dvec3(b.inv_mass, b.inv_mass, b.inv_inertia));
        let (j1, j2) = c.jacobian;
        let delta1 = m1_inv * j1 * impulse;
        let delta2 = m2_inv * j2 * impulse;
        trace!("Velocity delta 1: {delta1}, delta 2: {delta2}");

        a.vel.x += delta1.x;
        a.vel.y += delta1.y;
        a.omega += delta1.z;

        b.vel.x += delta2.x;
        b.vel.y += delta2.y;
        b.omega += delta2.z;
    }
}

impl Solver for SequentialImpulseSolver {
    #[instrument(level = "trace", skip_all)]
    fn solve(&self, particles: &mut [Particle], constraints: &mut [ConstraintData]) {
        for iter in 0..(self.iterations) {
            let span = trace_span!("Iteration", iter);
            let _enter = span.enter();
            for c in &mut *constraints {
                let (id_a, id_b) = c.constraint.get_ids();
                if id_a == id_b {
                    warn!("Constraint uses identical indices: {:?}", c);
                    continue;
                }
                let a = &particles[id_a];
                let b = &particles[id_b];
                // TODO: accumulate impulses over solver iterations to prevent
                // applying more impulse than necessary to achieve target velocity
                // see https://github.com/orbital-simulations/experimental/issues/51
                let impulse = self.find_impulse(a, b, c);
                let [a, b] = particles
                    .get_many_mut([id_a, id_b])
                    .expect("Invalid indices");
                self.apply(a, b, c, impulse);
            }
        }
    }
}
