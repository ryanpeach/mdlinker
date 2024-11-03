//! Code used in multiple test folders
use std::{path::PathBuf, str::FromStr};

use mdlinker::{config, lib};

/// Runs the library and generates the [`mdlinker::OutputReport`]
#[must_use]
pub fn get_report(paths: &[String]) -> mdlinker::OutputReport {
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
