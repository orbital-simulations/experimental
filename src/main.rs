use comfy::{simple_game, EngineContext, RED, GameLoop, egui, draw_circle, vec2, draw_text, WHITE, TextAlign};

simple_game!("egui example", update);

fn update(_c: &mut EngineContext) {
    egui::Window::new("Simple egui window")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(egui(), |ui| {
            if ui.button("hello").hovered() {
                ui.colored_label(RED.egui(), "from egui");
            } else {
                ui.label("from egui");
            }
        });

    draw_circle(vec2(0.0, 5.0), 0.5, RED * 5.0, 0);

    draw_text(
        "Nice red glowing circle with the help of HDR bloom",
        vec2(0.0, -2.0),
        WHITE,
        TextAlign::Center,
    );
}
