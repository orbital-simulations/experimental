use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn setup() -> color_eyre::eyre::Result<()> {
    let fmt_layer = fmt::layer().pretty();
    let filter_layer = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();
    color_eyre::install()
}
