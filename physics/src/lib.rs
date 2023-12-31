use geometry::Contact;
use glam::DVec2;

mod draw;
mod geometry;

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
            c1.collide_with_circle(&c2).into_iter().collect()
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
    // that has already occurred.
    //
    // For dynamic contacts it should reflect the initial velocity to conserve energy, possibly
    // divided by restitution factor to account for inelastic collisions.
    fn resolve_collisions(&mut self, collisions: &[Collision]) {
        for _ in 0..self.solver_iterations {
            for _col in collisions {
                // compute and apply momenta to fix the collision
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
