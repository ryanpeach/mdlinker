//! Code used in multiple test folders
use std::path::PathBuf;

use mdlinker::{
    config::{cli::Config as CliConfig, file::Config as FileConfig, Config},
    lib,
};

use std::sync::Once;

static INIT: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
fn setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

/// Runs the library and generates the [`mdlinker::OutputReport`]
#[must_use]
pub fn get_report(paths: &[PathBuf], config: Option<Config>) -> mdlinker::OutputReport {
    setup();
    let config: Config = match config {
        None => Config::builder()
            .files(paths.to_vec())
            .cli_config(CliConfig::default())
            .file_config(FileConfig::default())
            .build(),
        Some(config) => config,
    };

    lib(&config).expect("There should have been no error.")
}
