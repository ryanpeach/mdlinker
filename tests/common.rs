//! Code used in testing and in lib usage to get the report programmatically.
use std::path::PathBuf;

use mdlinker::{
    config::{cli::Config as CliConfig, file::Config as FileConfig, Config},
    lib,
};

/// Runs the library and generates the [`mdlinker::OutputReport`]
#[must_use]
pub fn get_report(paths: &[PathBuf], config: Option<Config>) -> mdlinker::OutputReport {
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
