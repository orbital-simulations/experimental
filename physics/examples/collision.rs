use physics::{
    scenarios::{self, Scenario as _},
    Engine,
};

mod shared;

#[derive(Default)]
pub struct GameState {
    engine: Engine,
}

impl GameState {
    fn setup(&mut self) {
        self.engine = scenarios::collision::Collision {}.create();
    }

    fn update(&mut self) {
        let dt = macroquad::time::get_frame_time();
        self.engine.step(dt as f64);
    }

    fn render(&self) {
        use shared::draw::Draw;
        self.engine.draw();
    }
}

#[macroquad::main("experimental")]
async fn main() {
    use macroquad::window::next_frame;
    shared::setup();
    let mut state = GameState::default();
    state.setup();

    loop {
        state.update();
        state.render();
        next_frame().await;
    }
}
