use game_engine::{game_engine_2_5d_parameters, GameEngine};
use glam::{DMat2, DVec2, Vec3};
use physics::{
    scenarios::{Collision, Scenario},
    Engine, Shape,
};
use renderer::{
    circle_rendering::CircleLine,
    colors::{RED, YELLOW},
    line_rendering::Line,
    transform::Transform,
    Renderer,
};
use tracing::debug;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winit::{event_loop::EventLoop, window::Window};

pub struct History {
    engine: Engine,
    history: Vec<(f64, Engine)>,
    frame: usize,
}

impl History {
    pub fn new(engine: Engine) -> Self {
        Self {
            engine: engine.clone(),
            history: vec![(0.0, engine)],
            frame: 0,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let max_frame = self.history.len() - 1;
        ui.add(egui::Slider::new(&mut self.frame, 0..=max_frame).text("frame"));
        let frame_delta = &self.history[self.frame].0;
        ui.label("Last delta: ");
        ui.label(format!("{:.4}", frame_delta));
    }

    pub fn is_last_frame(&self) -> bool {
        self.frame == self.history.len() - 1
    }

    pub fn update(&mut self, running: bool) {
        if !self.is_last_frame() {
            if running {
                self.history.truncate(self.frame + 1);
            }

            self.engine = self.history[self.frame].1.clone();
        }
    }

    pub fn step(&mut self, dt: f64) {
        self.engine.step(dt);
        self.history.push((dt, self.engine.clone()));
        self.frame += 1;
    }
}

pub struct GameState {
    running: bool,
    scenarios: Scenarios,
    active_scenario: usize,
    history: History,
}

fn setup(_game_engine: &mut GameEngine) -> GameState {
    GameState {
        running: true,
        scenarios: Scenarios::new(),
        active_scenario: 0,
        history: History::new(Collision {}.create()),
    }
}

struct Scenarios(Vec<Box<dyn Scenario>>);

impl Scenarios {
    fn new() -> Self {
        use physics::scenarios::*;
        let scenarios = vec![
            Box::new(Collision {}) as Box<dyn Scenario>,
            Box::new(InclinedFall {}) as Box<dyn Scenario>,
            Box::new(ManyParticles {}) as Box<dyn Scenario>,
            Box::new(Pendulum {}) as Box<dyn Scenario>,
            Box::new(Penetration {}) as Box<dyn Scenario>,
            Box::new(Resting {}) as Box<dyn Scenario>,
            Box::new(SimpleFall {}) as Box<dyn Scenario>,
            Box::new(Springs {}) as Box<dyn Scenario>,
        ];
        Scenarios(scenarios)
    }

    fn ui(&self, history: &mut History, active: &mut usize, ui: &mut egui::Ui) {
        ui.label("Scenarios");

        for (index, scenario) in self.0.iter().enumerate() {
            if ui.button(scenario.name()).clicked() {
                *history = History::new(scenario.create().clone());
                *active = index;
            }
        }
    }
}

fn update(state: &mut GameState, game_engine: &mut GameEngine) {
    // GUI
    let egui_context = game_engine.egui();
    egui::SidePanel::right("panel").show(egui_context, |ui| {
        ui.heading("Simulation controls");

        ui.add(
            egui::Slider::new(&mut state.history.engine.gravity.y, -100.0..=100.0).text("gravity"),
        );

        state.history.ui(ui);

        if !state.history.is_last_frame() {
            state.running = false;
        }

        let running_text = if state.running { "Pause" } else { "Play" };
        if ui.button(running_text).clicked() {
            state.running = !state.running;
        }

        state
            .scenarios
            .ui(&mut state.history, &mut state.active_scenario, ui);
    });

    // History management
    state.history.update(state.running);

    // Simulation
    if state.running {
        let scenario = &state.scenarios.0[state.active_scenario];
        scenario.update(&mut state.history.engine);
        let dt = game_engine.last_frame_delta as f64;
        state.history.step(dt);
    }
}

fn render(state: &GameState, renderer: &mut Renderer) {
    debug!("main render");
    for p in &state.history.engine.particles {
        match p.shape {
            Shape::Circle { radius } => {
                renderer.draw_circle_line(
                    &Transform::from_translation(&Vec3::new(p.pos.x as f32, p.pos.y as f32, 0.0)),
                    &CircleLine::new(radius as f32, RED, 3.0),
                );
                let direction = DMat2::from_angle(p.angle) * DVec2::X;
                let to = (p.pos + direction * radius).as_vec2();
                renderer.draw_line(&Line::new(
                    Vec3::new(p.pos.x as f32, p.pos.y as f32, 0.0),
                    Vec3::new(to.x, to.y, 0.0),
                    RED,
                    1.0,
                ));
            }
            Shape::HalfPlane { normal_angle } => {
                let extent = 10000.0;
                let tangent = DVec2::from_angle(normal_angle).perp();
                let from: DVec2 = p.pos + extent * tangent;
                let to: DVec2 = p.pos - extent * tangent;
                renderer.draw_line(&Line::new(
                    Vec3::new(from.x as f32, from.y as f32, 0.0),
                    Vec3::new(to.x as f32, to.y as f32, 0.0),
                    YELLOW,
                    3.0,
                ));
            }
            _ => {
                unimplemented!("Render unknown shape {:?}", p.shape)
            }
        }
    }
}

fn main() -> color_eyre::eyre::Result<()> {
    let fmt_layer = fmt::layer().pretty();
    let filter_layer = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();
    color_eyre::install()?;
    let event_loop = EventLoop::new()?;
    let window = Window::new(&event_loop)?;
    let (mut game_engine, event_loop) = pollster::block_on(GameEngine::new(
        event_loop,
        &window,
        game_engine_2_5d_parameters(),
    ))?;
    game_engine.run(event_loop, setup, &update, &render)?;
    Ok(())
}
