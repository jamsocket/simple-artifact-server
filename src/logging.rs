use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing() {
    // Initialize the tracing subscriber with a default configuration
    // that writes formatted spans and events to stdout
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_target(true)
        .init();
}
