use std::iter::repeat_with;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use glam::dvec2;
use physics::{Engine, Particle, Shape};
use rand::Rng;

fn init_circle_engine(num_particles: usize) -> Engine {
    let mut engine = Engine::default();
    let mut rng = rand::thread_rng();
    let pos_limit = 500.0;
    let vel_limit = 50.0;
    engine.particles.extend(
        repeat_with(|| Particle {
            mass: rng.gen_range(1.0..3.0),
            pos: dvec2(
                rng.gen_range(-pos_limit..pos_limit),
                rng.gen_range(-pos_limit..pos_limit),
            ),
            vel: dvec2(
                rng.gen_range(-vel_limit..vel_limit),
                rng.gen_range(-vel_limit..vel_limit),
            ),
            shape: Shape::Circle(10.),
            ..Default::default()
        })
        .take(num_particles),
    );
    engine
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_elem");
    for num_particles in [32, 64, 128, 256, 512] {
        let initial_engine = init_circle_engine(num_particles);
        group.bench_with_input(
            BenchmarkId::new("step many circles", num_particles),
            &num_particles,
            |b, _num_particles| {
                b.iter(|| {
                    let dt = 1.0 / 60.0;
                    let mut engine = initial_engine.clone();
                    engine.step(black_box(dt));
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
