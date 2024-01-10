use macroquad::{
    camera::{set_camera, Camera2D},
    math::Rect,
    window::{screen_height, screen_width},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub mod draw;

pub fn setup() {
    let fmt_layer = fmt::layer().compact();
    let filter_layer = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();
    let (w, h) = (screen_width(), screen_height());
    let camera = Camera2D::from_display_rect(Rect::new(-w / 2.0, -h / 2.0, w, h));
    set_camera(&camera);
}
