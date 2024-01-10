#![feature(get_many_mut)]
use constraint::{CollisionConstraint, Constraint, ConstraintEnum};
use geometry::{Capsule, Circle, HalfPlane};
use glam::DVec2;
use solver::{ConstraintData, SequentialImpulseSolver, Solver};
use tracing::{instrument, trace, trace_span};

pub mod constraint;

pub mod geometry;

pub mod solver;

/// A representation of a rigid body possessing geometry (`pos`, `angle`, `shape`),
/// kinematics (`vel`, `omega`) and dynamics (`inv_mass`, `force`, `inv_inertia`, `torque`).
#[derive(Clone, Debug)]
pub struct Particle {
    /// A non-negative number that represents `mass = 1.0 / inv_mass` if it is positive
    /// and an infinite mass (i.e. immovable object) when it is zero.
    pub inv_mass: f64,
    /// Position
    pub pos: DVec2,
    /// Velocity
    pub vel: DVec2,
    /// Force to be applied specifically to this particle during the next simulation step.
    /// Note that global forces such as gravity can be specified with `Engine::gravity`.
    pub force: DVec2,
    /// A non-negative number representing inverse of object's moment of inertia.
    /// Zero corresponds to infinite inertia (i.e. immovable object).
    /// Moment inertia depends on object's geometry and mass density distribution.
    /// TODO: provide helper functions to calculate inertia for common shapes with uniform density.
    /// see https://github.com/orbital-simulations/experimental/issues/56
    pub inv_inertia: f64,
    /// Orientation
    pub angle: f64,
    /// Angular velocity
    pub omega: f64,
    /// Force moment to be applied specifically to this particle during the next simulation step.
    /// Represents an angular action that causes change in angular velocity.
    pub torque: f64,
    /// Geometry of the rigid body.
    pub shape: Shape,
}

impl Particle {
    pub fn new(inv_mass: f64, inv_inertia: f64, shape: Shape) -> Particle {
        Particle {
            inv_mass,
            pos: DVec2::ZERO,
            vel: DVec2::ZERO,
            force: DVec2::ZERO,
            inv_inertia,
            angle: 0.0,
            omega: 0.0,
            torque: 0.0,
            shape,
        }
    }
}

impl Particle {
    pub fn to_geometry_shape(&self) -> geometry::Shape {
        match self.shape {
            Shape::Circle { radius } => geometry::Shape::Circle(Circle {
                pos: self.pos,
                radius,
            }),
            Shape::Capsule { length, radius } => {
                geometry::Shape::Capsule(Capsule::new(self.pos, self.angle, length, radius))
            }
            Shape::HalfPlane { normal_angle } => geometry::Shape::HalfPlane(HalfPlane {
                pos: self.pos,
                normal_angle,
            }),
        }
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self::new(1.0, 1.0, Shape::Circle { radius: 1.0 })
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Shape {
    Circle {
        radius: f64,
    },
    Capsule {
        length: f64,
        radius: f64,
    },
    HalfPlane {
        /// normal's angle with the x-axis in counter-clock-wise direction, in radians
        normal_angle: f64,
    },
}

#[derive(Clone, Debug)]
pub struct Engine {
    pub particles: Vec<Particle>,
    pub constraints: Vec<ConstraintEnum>,
    pub gravity: DVec2,
    pub solver_iterations: usize,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            particles: Default::default(),
            constraints: Default::default(),
            gravity: Default::default(),
            solver_iterations: 10,
        }
    }
}

const STATIC_SPEED_FACTOR: f64 = 2.0;

impl Engine {
    #[instrument(level = "trace", skip_all)]
    pub fn detect_collisions(&self) -> Vec<CollisionConstraint> {
        let mut collisions = vec![];
        for (i, a) in self.particles.iter().enumerate() {
            for (j, b) in self.particles.iter().enumerate() {
                if i >= j {
                    continue;
                }

                let contacts = a
                    .to_geometry_shape()
                    .test_overlap(&b.to_geometry_shape())
                    .into_iter()
                    .map(|contact| CollisionConstraint::new(i, j, contact, true));
                collisions.extend(contacts)
            }
        }
        collisions
    }

    // TODO: resolve_collisions is now obsolete but maybe it could be useful
    // for some comparison tests and some documentation might be salvagable.
    // see https://github.com/orbital-simulations/experimental/issues/50

    // The goal of collision resolution is to solve all the constraints between particles.
    // These constraints can be explicitly set by the user (TBI) but they can also arise
    // implicitly to avoid penetration.
    //
    // The penetration constraints add requirements for relative normal velocity at contact points.
    // For static contacts this velocity should be non-negative, or positive to offset penetration
    // that has already occurred. For dynamic contacts it should reflect the initial velocity
    // to conserve energy, possibly multiplied by restitution factor to account for inelastic collisions.
    //
    // We use sequential impulse solving to resolve constraints one-by-one by applying impulses
    // and iterating to hopefully converge to a globally reasonable solution.
    //
    // Linear constraints could be solved with linear algebra and it might make sense to do so
    // in certain scenarios but there are two problems with this approach in general:
    // 1. Inverting big matrices is slow, so an approximate iterative solution might still be preferred.
    // 2. Many constraints are non-linear; this applies in particular to penetration constraints:
    //    if one treats them as linear, they become "sticky", whereas we only want them to be repulsive.
    #[instrument(level = "trace", skip_all)]
    fn resolve_collisions(&mut self, collisions: &[CollisionConstraint]) {
        use glam::{dvec3, DMat3};

        for iter in 0..self.solver_iterations {
            let span = trace_span!("Iteration", iter);
            let _enter = span.enter();
            for col in collisions {
                // The geometry of the contact is described by a 6D row-vector called 'Jacobian':
                // J = (-n.x, -n.y, -n \cross r_1, n.x, n.y, n \cross r_2)
                // where `n` is the contact normal pointing from the first to the second particle
                // and `r_i` is an "arm" vector from the `i`'s center of mass to the contact point.
                //
                // If we also aggregate all the velocities as `V^T = (v_1.x, v_1.y, omega_1, v_2.x, v_2.y, omega_2)``
                // we can use the Jacobian to easily compute relative normal velocity as:
                // v_rel = J * V.
                //
                // Similarly, Jacobian can tell us how the velocities change
                // when we apply an impulse `p = \lambda n` alongside the contact normal:
                // \Delta V = M^{-1} * J^T * \lambda
                // where `M`` is a matrix of masses and inertias `M = diag(m_1, m_1, I_1, m_2, m_2, I_2)`.
                //
                // From this we can compute the resulting relative velocity:
                // v'_rel = J * (V + \Delta V) = v_rel + J * M^{-1} * J^T * \lambda
                // If we know the desired relative velocity, e.g. for
                // static contacts v'_rel = 0 and for dynamic contacts v'_rel = -restitution * v_rel,
                // we can compute `\lambda`:
                // \lambda = (v'_rel - v_rel) / (J * M^{-1} * J^T)
                //
                // Since `M`` is diagonal we can solve everything manually without a linear algebra package.
                // In contrast with description above we replace 6D vectors with two 3D vectors.

                let p1 = &self.particles[col.id_a];
                let p2 = &self.particles[col.id_b];
                let r1 = col.contact.pos - p1.pos;
                let r2 = col.contact.pos - p2.pos;
                let n = col.contact.normal;
                // TODO: Jacobians don't change during solving and should be precomputed
                let j1 = dvec3(-n.x, -n.y, -n.perp_dot(r1));
                let j2 = dvec3(n.x, n.y, n.perp_dot(r2));
                let v1 = dvec3(p1.vel.x, p1.vel.y, p1.omega);
                let v2 = dvec3(p2.vel.x, p2.vel.y, p2.omega);
                let v_rel = j1.dot(v1) + j2.dot(v2);
                trace!("Velocity 1: {v1}, velocity 2: {v2}, relative velocity: {v_rel}");
                // TODO: this is not correct, we need to check general constraint satisfaction
                // Objects are already separating, nothing to do here.
                if v_rel >= 0.0 {
                    // TODO: accumulate impulses over solver iterations to prevent
                    // applying more impulse than necessary to achieve target v_rel
                    continue;
                }
                let m1_inv = DMat3::from_diagonal(dvec3(p1.inv_mass, p1.inv_mass, p1.inv_inertia));
                let m2_inv = DMat3::from_diagonal(dvec3(p2.inv_mass, p2.inv_mass, p2.inv_inertia));
                // Supporting only dynamic contacts with ellastic collision for now
                let restitution = 1.0;
                let lambda =
                    (-restitution - 1.0) * v_rel / (j1.dot(m1_inv * j1) + j2.dot(m2_inv * j2));

                let delta_1 = m1_inv * j1 * lambda;
                let p1 = &mut self.particles[col.id_a];
                p1.vel.x += delta_1.x;
                p1.vel.y += delta_1.y;
                p1.omega += delta_1.z;

                let delta_2 = m2_inv * j2 * lambda;
                trace!("Lambda {lambda}, delta 1: {delta_1}, delta 2: {delta_2}");
                let p2 = &mut self.particles[col.id_b];
                p2.vel.x += delta_2.x;
                p2.vel.y += delta_2.y;
                p2.omega += delta_2.z;
            }
        }
    }

    /// Simulates movement of particles for a duration `dt`.
    /// Besides free movement we also apply forces, satisfy constraints and resolve collisions.
    pub fn step(&mut self, dt: f64) {
        // 1. Update velocities from forces
        for p in &mut self.particles {
            let force = self.gravity + p.force;
            let acc = force * p.inv_mass;
            p.vel += dt * acc;

            let alpha = p.torque * p.inv_inertia;
            p.omega += dt * alpha;
        }

        // TODO: should we predict positions using the updated velocities before detecting collisions?
        // see https://github.com/orbital-simulations/experimental/issues/55

        // 2. Detect collisions
        let collision_constraints: Vec<_> = self
            .detect_collisions()
            .into_iter()
            .filter_map(|mut c| {
                let a = &self.particles[c.id_a];
                let b = &self.particles[c.id_b];
                // NOTE: we treat collisions with low relative velocity as static, i.e. we do not
                // conserve energy, we only prevent penetration.
                // TODO: a better approach might be to track collisions over multiple frames
                // and consider only new ones as dynamic,
                // see https://github.com/orbital-simulations/experimental/issues/58
                let static_speed_limit = STATIC_SPEED_FACTOR * self.gravity.length() * dt;
                let v_rel = c.relative_velocity(a, b);
                if v_rel.abs() < static_speed_limit {
                    c.dynamic = false;
                }
                if v_rel < static_speed_limit {
                    Some(ConstraintEnum::Collision(c))
                } else {
                    // TODO: this is probably not correct, we should use all collisions
                    // in the solver even though the objects are separating initially,
                    // since relative velocity might change during solving and we will fail
                    // to take this interaction into account.
                    // That said, right now this gives the best simulation results.
                    // Having persistent contacts would probably fix this,
                    // see https://github.com/orbital-simulations/experimental/issues/58
                    None
                }
            })
            .collect();

        // Prepare both collision and user constraints for the solver
        let mut constraint_data: Vec<_> = self
            .constraints
            .iter()
            .chain(collision_constraints.iter())
            .map(|c| ConstraintData::from_constraint(c, &self.particles, dt))
            .collect();

        // 3. Solve all constraints
        let solver = SequentialImpulseSolver {
            dt,
            iterations: self.solver_iterations,
        };
        solver.solve(&mut self.particles, &mut constraint_data);

        // 4. Update positions & reset forces
        for p in &mut self.particles {
            p.pos += dt * p.vel;
            p.force = DVec2::ZERO;

            p.angle += dt * p.omega;
            p.torque = 0.0;
        }
    }
}
