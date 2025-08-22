use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init_logging(verbosity: u8) -> anyhow::Result<()> {
    let level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(EnvFilter::new(level))
        .with(fmt::layer())
        .with(ErrorLayer::default())
        .try_init()?;

    Ok(())
}
