use macroquad::{time::get_frame_time, window::next_frame};
use physics::{
    scenarios::{Penetration, Scenario},
    Engine,
};

mod shared;

#[derive(Default)]
pub struct GameState {
    engine: Engine,
}

impl GameState {
    fn setup(&mut self) {
        self.engine = Penetration {}.create();
    }

    fn update(&mut self) {
        let dt = get_frame_time();
        self.engine.step(dt as f64);
    }

    fn render(&self) {
        use shared::draw::Draw;
        self.engine.draw();
    }
}

#[macroquad::main("experimental")]
async fn main() {
    shared::setup();
    let mut state = GameState::default();
    state.setup();

    loop {
        state.update();
        state.render();
        next_frame().await;
    }
}
