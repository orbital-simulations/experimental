use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use winit::{event_loop::EventLoop, window::Window};

pub fn setup() -> color_eyre::eyre::Result<(EventLoop<()>, Window)> {
    let fmt_layer = fmt::layer().pretty();
    let filter_layer = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();
    color_eyre::install()?;

    let event_loop = EventLoop::new().expect("Can't create the event loop");
    let window = Window::new(&event_loop).expect("Can't create the window");

    Ok((event_loop, window))
}
