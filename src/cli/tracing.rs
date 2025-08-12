use anyhow::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init_logging(verbosity: u8) -> Result<()> {
    miette::set_panic_hook();

    let level = match verbosity {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };

    tracing_subscriber::registry()
        .with(EnvFilter::new(level))
        .with(fmt::layer())
        .with(ErrorLayer::default())
        .try_init()?;

    Ok(())
}
