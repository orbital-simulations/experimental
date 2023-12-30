//use macroquad::prelude::*;
use glam::{dvec2, DVec2};
use macroquad::{color::RED, shapes::draw_circle, time::get_frame_time, window::next_frame};
use physics::physics::{Engine, Particle};

pub struct GameState {
    engine: Engine,
}

const GRAVITY: DVec2 = DVec2::new(0.0, 9.81);

impl GameState {
    fn setup(&mut self) {
        self.engine.particles = vec![Particle {
            mass: 1.0,
            pos: dvec2(400.0, 200.0),
            vel: DVec2::ZERO,
        }]
    }

    fn update(&mut self) {
        let dt = get_frame_time();

        self.engine.step(dt as f64);
    }

    fn render(&self) {
        for p in &self.engine.particles {
            let pos = p.pos.as_vec2();
            draw_circle(pos.x, pos.y, 5.0, RED);
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            engine: Engine::new(GRAVITY),
        }
    }
}

#[macroquad::main("experimental")]
async fn main() {
    let mut state = GameState::default();
    state.setup();

    loop {
        state.update();
        state.render();
        next_frame().await;
    }
}
