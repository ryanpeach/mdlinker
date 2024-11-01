//! Code used in multiple test folders
use std::{path::PathBuf, str::FromStr};

use mdlinker::{config, lib};

/// Runs the library and generates the [`mdlinker::OutputReport`]
#[must_use]
pub fn get_report() -> mdlinker::OutputReport {
    let config = config::Config::builder()
        .directories(vec![
            PathBuf::from_str("./assets/pages").expect("This path exists at compile time.")
        ])
        .build();

    lib(&config).expect("Shouldn't be a problem with the linter itself")
}
