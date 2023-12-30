use comfy::{
    draw_circle, draw_text, egui, frame_time, simple_game, vec2, EngineContext, EngineState,
    GameConfig, GameLoop, TextAlign, RED, WHITE,
};
use glam::{dvec2, DVec2};
use physics::{Engine, Particle, Shape};

simple_game!("experimental", GameState, config, setup, update);

pub struct GameState {
    engine: Engine,
}

impl GameState {
    pub fn new(_c: &EngineState) -> Self {
        Self {
            engine: Engine::default(),
        }
    }
}

fn config(config: GameConfig) -> GameConfig {
    GameConfig {
        vsync_enabled: false,
        target_framerate: 120,
        ..config
    }
}

const GRAVITY: DVec2 = DVec2::new(0.0, -9.81);

fn setup(state: &mut GameState, _c: &mut EngineContext) {
    state.engine.gravity = GRAVITY;
    state.engine.particles = vec![Particle {
        mass: 1.0,
        pos: dvec2(0.0, 10.0),
        vel: DVec2::ZERO,
        shape: Shape::Circle(0.5),
    }]
}

fn update(state: &mut GameState, _c: &mut EngineContext) {
    let dt = frame_time();

    egui::Window::new("Simple egui window")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(egui(), |ui| {
            if ui.button("hello").hovered() {
                ui.colored_label(RED.egui(), "from egui");
            } else {
                ui.label("from egui");
            }
        });

    state.engine.step(dt as f64);

    for p in &state.engine.particles {
        if let Shape::Circle(radius) = p.shape {
            draw_circle(p.pos.as_vec2(), radius as f32, RED * 5.0, 0);
        }
    }

    draw_text(
        "Nice red glowing circle with the help of HDR bloom",
        vec2(0.0, -2.0),
        WHITE,
        TextAlign::Center,
    );
}
