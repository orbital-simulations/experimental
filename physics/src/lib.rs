use geometry::Contact;
use glam::DVec2;

pub mod geometry;

/// A representation of a rigid body possessing geometry (`pos`, `angle`, `shape`),
/// kinematics (`vel`, `omega`) and dynamics (`mass`, `force`, `inertia`, `torque`).
pub struct Particle {
    pub mass: f64,
    pub pos: DVec2,
    pub vel: DVec2,
    pub force: DVec2,
    pub inertia: f64,
    pub angle: f64,
    pub omega: f64,
    pub torque: f64,
    pub shape: Shape,
}

impl Particle {
    pub fn new(mass: f64, inertia: f64, shape: Shape) -> Particle {
        Particle {
            mass,
            pos: DVec2::ZERO,
            vel: DVec2::ZERO,
            force: DVec2::ZERO,
            inertia,
            angle: 0.0,
            omega: 0.0,
            torque: 0.0,
            shape,
        }
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self::new(1.0, 1.0, Shape::Circle(1.0))
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Shape {
    Circle(f64),
}

pub struct Engine {
    pub particles: Vec<Particle>,
    pub gravity: DVec2,
    pub solver_iterations: usize,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            particles: Default::default(),
            gravity: Default::default(),
            solver_iterations: 10,
        }
    }
}

#[allow(dead_code)]
pub struct Collision {
    pub id_a: usize,
    pub id_b: usize,
    pub contact: Contact,
}

impl Collision {
    fn new(a: usize, b: usize, contact: Contact) -> Collision {
        Collision {
            id_a: a,
            id_b: b,
            contact,
        }
    }
}

fn get_contacts(a: &Particle, b: &Particle) -> Vec<Contact> {
    match (&a.shape, &b.shape) {
        (Shape::Circle(r1), Shape::Circle(r2)) => {
            let c1 = geometry::Circle {
                pos: a.pos,
                radius: *r1,
            };
            let c2 = geometry::Circle {
                pos: b.pos,
                radius: *r2,
            };
            c1.test_contact_with_circle(&c2).into_iter().collect()
        }
    }
}

impl Engine {
    pub fn detect_collisions(&self) -> Vec<Collision> {
        let mut collisions = vec![];
        for (i, a) in self.particles.iter().enumerate() {
            for (j, b) in self.particles.iter().enumerate() {
                if i >= j {
                    continue;
                }

                let new_collisions = get_contacts(a, b)
                    .into_iter()
                    .map(|contact| Collision::new(i, j, contact));
                collisions.extend(new_collisions)
            }
        }
        collisions
    }

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
    fn resolve_collisions(&mut self, collisions: &[Collision]) {
        use glam::{dvec3, DMat3};

        for _ in 0..self.solver_iterations {
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
                // Objects are already separating, nothing to do here.
                if v_rel >= 0.0 {
                    // TODO: accumulate impulses over solver iterations to prevent
                    // applying more impulse than necessary to achieve target v_rel
                    continue;
                }
                // TODO: consider storing inverse masses in `Particle`
                let m1_inv =
                    DMat3::from_diagonal(dvec3(1.0 / p1.mass, 1.0 / p1.mass, 1.0 / p1.inertia));
                let m2_inv =
                    DMat3::from_diagonal(dvec3(1.0 / p2.mass, 1.0 / p2.mass, 1.0 / p2.inertia));
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
                let p2 = &mut self.particles[col.id_b];
                p2.vel.x += delta_2.x;
                p2.vel.y += delta_2.y;
                p2.omega += delta_2.z;
            }
        }
    }

    pub fn step(&mut self, dt: f64) {
        // 1. Update velocities from forces
        for p in &mut self.particles {
            let force = self.gravity + p.force;
            let acc = force / p.mass;
            p.vel += dt * acc;

            let alpha = p.torque / p.inertia;
            p.omega += dt * alpha;
        }

        // TODO: should we predict positions using the updated velocities before detecting collisions?

        // 2. Detect collisions
        let collisions = self.detect_collisions();

        // 3. Resolve collisions by updating velocities
        self.resolve_collisions(&collisions);

        // 4. Update positions & reset forces
        for p in &mut self.particles {
            p.pos += dt * p.vel;
            p.force = DVec2::ZERO;

            p.angle += dt * p.omega;
            p.torque = 0.0;
        }
    }
}
