use mdlinker::config;
use mdlinker::lib;
use miette::{miette, Result};

/// Really just a wrapper that loads the config and passes it to the main library function
fn main() -> Result<()> {
    env_logger::init();

    // Load the configuration
    let config = config::Config::new().map_err(|e| miette!(e))?;

    lib(&config)
}
