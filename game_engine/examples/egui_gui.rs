use game_engine::{game_engine_2_5d_parameters, GameEngine, RenderingBackend, Scene};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winit::{event_loop::EventLoop, window::Window};

pub struct GameState {
    name: String,
    age: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

fn setup(_game_engine: &mut GameEngine) -> GameState {
    GameState::default()
}

fn update(state: &mut GameState, game_engine: &mut GameEngine) {
    let egui_context = game_engine.egui();

    egui::CentralPanel::default().show(egui_context, |ui| {
        ui.heading("My egui Application");
        ui.horizontal(|ui| {
            let name_label = ui.label("Your name: ");
            ui.text_edit_singleline(&mut state.name)
                .labelled_by(name_label.id);
        });
        ui.add(egui::Slider::new(&mut state.age, 0..=120).text("age"));
        if ui.button("Click each year").clicked() {
            state.age += 1;
        }
        ui.label(format!("Hello '{}', age {}", state.name, state.age));
    });
}

fn render(_state: &GameState, _scene: &mut Scene) {}

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
        game_engine_2_5d_parameters(RenderingBackend::Native),
    ))?;
    game_engine.run(event_loop, setup, &update, &render)?;
    Ok(())
}
