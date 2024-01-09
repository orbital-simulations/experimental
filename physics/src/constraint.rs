use core::fmt;

use dyn_clone::DynClone;
use glam::{dvec3, DVec3};
use tracing::trace;

use crate::{geometry::Contact, Particle};

#[derive(Clone, Debug)]
pub enum ConstraintEnum {
    Distance(DistanceConstraint),
    Collision(CollisionConstraint),
    Custom(Box<dyn Constraint>),
}

macro_rules! dispatch_constraint {
    ($self: ident, $method: ident, $( $arg: ident),* ) => {
        match $self {
            ConstraintEnum::Distance(c) => c.$method($( $arg ),*),
            ConstraintEnum::Collision(c) => c.$method($( $arg ),*),
            ConstraintEnum::Custom(c) => c.$method($( $arg ),*)
        }
    };
}

impl Constraint for ConstraintEnum {
    fn get_ids(&self) -> (usize, usize) {
        dispatch_constraint!(self, get_ids,)
    }

    fn is_satisfied(&self, a: &Particle, b: &Particle, dt: f64) -> bool {
        dispatch_constraint!(self, is_satisfied, a, b, dt)
    }

    fn value(&self, a: &Particle, b: &Particle) -> f64 {
        dispatch_constraint!(self, value, a, b)
    }

    fn jacobian(&self, a: &Particle, b: &Particle) -> (DVec3, DVec3) {
        dispatch_constraint!(self, jacobian, a, b)
    }

    fn target_velocity(&self, a: &Particle, b: &Particle, dt: f64) -> f64 {
        dispatch_constraint!(self, target_velocity, a, b, dt)
    }
}

/// An equality constraint is defined by a function C(a, b) between two particles.
/// We want C(a, b) = 0 to be satisfied and to achieve that we need to be able to evaluate
/// the function so that we know how close we are, as well as its gradient
/// with respect to both particles so that we know how to improve the situation.
///
/// Writing C(a(t), b(t)) as a function of time, one can define this gradient,
/// also called 'Jacobian', as dC/dt = J * (da/dt, db/dt) = J * V.
pub trait Constraint: fmt::Debug + DynClone {
    fn get_ids(&self) -> (usize, usize);

    fn is_satisfied(&self, a: &Particle, b: &Particle, dt: f64) -> bool;

    fn value(&self, a: &Particle, b: &Particle) -> f64;

    fn target_velocity(&self, a: &Particle, b: &Particle, dt: f64) -> f64;

    fn jacobian(&self, a: &Particle, b: &Particle) -> (DVec3, DVec3);

    fn relative_velocity(&self, a: &Particle, b: &Particle) -> f64 {
        // TODO: jacobian should be precomputed
        // This is only needed by `is_satisfied`, so might get fixed with
        // https://github.com/orbital-simulations/experimental/issues/49
        let (j1, j2) = self.jacobian(a, b);
        let v1 = dvec3(a.vel.x, a.vel.y, a.omega);
        let v2 = dvec3(b.vel.x, b.vel.y, b.omega);
        let v_rel = j1.dot(v1) + j2.dot(v2);
        trace!("Velocity 1: {v1}, velocity 2: {v2}, relative velocity: {v_rel}");
        v_rel
    }
}

dyn_clone::clone_trait_object!(Constraint);

#[derive(Clone, Debug)]
pub struct DistanceConstraint {
    pub id_a: usize,
    pub id_b: usize,
    pub distance: f64,
}

impl DistanceConstraint {
    pub fn new(id_a: usize, id_b: usize, distance: f64) -> DistanceConstraint {
        DistanceConstraint {
            id_a,
            id_b,
            distance,
        }
    }
}

const CONSTRAINT_TOLERANCE: f64 = 1e-6;

impl Constraint for DistanceConstraint {
    fn get_ids(&self) -> (usize, usize) {
        (self.id_a, self.id_b)
    }

    // TODO: move this to solver, see https://github.com/orbital-simulations/experimental/issues/49
    fn is_satisfied(&self, a: &Particle, b: &Particle, dt: f64) -> bool {
        let velocity_diff = self.target_velocity(a, b, dt) - self.relative_velocity(a, b);
        // TODO: precision, see https://github.com/orbital-simulations/experimental/issues/49
        self.value(a, b).abs() < CONSTRAINT_TOLERANCE && velocity_diff.abs() < CONSTRAINT_TOLERANCE
    }

    fn value(&self, a: &Particle, b: &Particle) -> f64 {
        (b.pos - a.pos).length() - self.distance
    }

    // To first order C(t+dt) ~ C(t) + dC/dt * dt = C(t) + J * v * dt = C(t) + v_rel * dt
    // If we want to achieve C(t+dt) = 0 we get v_rel = -C(t) / dt
    fn target_velocity(&self, a: &Particle, b: &Particle, dt: f64) -> f64 {
        -self.value(a, b) / dt
    }

    fn jacobian(&self, a: &Particle, b: &Particle) -> (DVec3, DVec3) {
        let diff = b.pos - a.pos;
        let distance = diff.length();
        // TODO: decide how to handle coinciding particles
        // see https://github.com/orbital-simulations/experimental/issues/54
        if distance < CONSTRAINT_TOLERANCE {
            unimplemented!("Constraints between coinciding particles")
        }
        let j1 = -diff / distance;
        let j2 = diff / distance;
        (dvec3(j1.x, j1.y, 0.0), dvec3(j2.x, j2.y, 0.0))
    }
}

#[derive(Clone, Debug)]
pub struct CollisionConstraint {
    pub id_a: usize,
    pub id_b: usize,
    pub contact: Contact,
}

impl CollisionConstraint {
    pub fn new(a: usize, b: usize, contact: Contact) -> CollisionConstraint {
        CollisionConstraint {
            id_a: a,
            id_b: b,
            contact,
        }
    }
}

impl Constraint for CollisionConstraint {
    fn get_ids(&self) -> (usize, usize) {
        (self.id_a, self.id_b)
    }

    fn is_satisfied(&self, a: &Particle, b: &Particle, dt: f64) -> bool {
        let velocity_diff = self.target_velocity(a, b, dt) - self.relative_velocity(a, b);
        // TODO: precision, see https://github.com/orbital-simulations/experimental/issues/49
        self.value(a, b) > -CONSTRAINT_TOLERANCE && velocity_diff.abs() < CONSTRAINT_TOLERANCE
    }

    fn value(&self, _a: &Particle, _b: &Particle) -> f64 {
        // TODO: write out the full constraint function and check that it equals to separation
        // see https://github.com/orbital-simulations/experimental/issues/50
        self.contact.separation
    }

    fn target_velocity(&self, a: &Particle, b: &Particle, _dt: f64) -> f64 {
        // TODO: compute restitution from some particle properties
        // see https://github.com/orbital-simulations/experimental/issues/53
        let restitution = 1.0;
        let v_rel = self.relative_velocity(a, b);
        -restitution * v_rel
    }

    fn jacobian(&self, a: &Particle, b: &Particle) -> (DVec3, DVec3) {
        let r1 = self.contact.pos - a.pos;
        let r2 = self.contact.pos - b.pos;
        let n = self.contact.normal;
        let j1 = dvec3(-n.x, -n.y, -n.perp_dot(r1));
        let j2 = dvec3(n.x, n.y, n.perp_dot(r2));
        (j1, j2)
    }
}
