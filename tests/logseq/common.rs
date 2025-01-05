//! Code used in multiple test folders
use std::{path::PathBuf, str::FromStr};

use mdlinker::{config::Config, lib};

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
pub fn get_report(paths: &[String], config: Option<Config>) -> mdlinker::OutputReport {
    setup();
    let config: Config = match config {
        None => {
            let paths: Vec<PathBuf> = paths
                .iter()
                .map(|path| PathBuf::from_str(path).expect("This path exists at compile time."))
                .collect();
            Config::builder()
                .pages_directory(paths[0].clone())
                .other_directories(paths[1..].to_vec())
                .build()
        }
        Some(config) => config,
    };

    lib(&config).expect("There should have been no error.")
}
