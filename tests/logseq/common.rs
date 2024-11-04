//! Code used in multiple test folders
use std::{path::PathBuf, str::FromStr};

use mdlinker::{config, lib};

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
pub fn get_report(paths: &[String]) -> mdlinker::OutputReport {
    setup();
    let config = config::Config::builder()
        .directories(
            paths
                .iter()
                .map(|path| PathBuf::from_str(path).expect("This path exists at compile time."))
                .collect(),
        )
        .build();

    lib(&config).expect("There should have been no error.")
}
