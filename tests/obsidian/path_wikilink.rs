#[path = "../common.rs"]
mod common;
use common::get_report;

use config::cli::Config as CliConfig;
use config::file::Config as FileConfig;
use glob::glob;
use log::info;
use mdlinker::rules::similar_filename::SimilarFilename;
use mdlinker::{config, lib};
use regex::Regex;
use std::path::PathBuf;

static PATHS: std::sync::LazyLock<Vec<PathBuf>> = std::sync::LazyLock::new(|| {
    glob("./tests/obsidian/path_wikilink/**/*.md")
        .expect("This is a constant")
        .map(|p| p.expect("This is a constant"))
        .collect()
});

#[test]
fn test_path_wikilink_no_errors() {}

#[test]
fn test_path_wikilink_has_8_wikilinks_all_pointing_to_baz_md() {
    info!("test_path_wikilink_has_8_wikilinks_all_pointing_to_baz_md");
    let config = config::Config::builder()
        .files(PATHS.to_vec())
        .file_config(FileConfig::default())
        .cli_config(CliConfig::default())
        .build();

    if let Err(e) = lib(&config) {
        panic!("There should have been no error. Found: {e}");
    }

    let report = get_report(PATHS.as_slice(), Some(config));
    report
}
