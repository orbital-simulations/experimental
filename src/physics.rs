use glam::DVec2;

pub struct Particle {
    pub mass: f64,
    pub pos: DVec2,
    pub vel: DVec2,
}

pub struct Engine {
    pub particles: Vec<Particle>,
    gravity: DVec2,
}

impl Engine {
    pub fn new(gravity: DVec2) -> Engine {
        Engine {
            particles: vec![],
            gravity,
        }
    }

    pub fn step(&mut self, dt: f64) {
        for p in &mut self.particles {
            let acc = self.gravity / p.mass;
            p.vel += dt * acc;
            p.pos += dt * p.vel;
        }
    }
}
