use crate::common::get_report;
use config::cli::Config as CliConfig;
use config::file::Config as FileConfig;
use lazy_static::lazy_static;
use log::info;
use mdlinker::rules::similar_filename::SimilarFilename;
use mdlinker::{config, lib};
use regex::Regex;
use std::{path::PathBuf, str::FromStr};

lazy_static! {
    static ref PATHS: Vec<String> =
        vec!["./tests/logseq/similar_filename/assets/pages".to_string(),];
}

/// [`foo.md`](./assets/logseq/pages/foo.md) and [`foo___bar.md`](./assets/logseq/pages/foo___bar.md) should not conflict
/// because the word `foo` in `foo/bar` is just a properly used group name.
#[test]
fn groups_first_element_same() {
    info!("groups_first_element_same");
    let config = config::Config::builder()
        .pages_directory(
            PathBuf::from_str("./tests/logseq/similar_filename/assets/pages")
                .expect("This is a constant"),
        )
        .file_config(FileConfig::default())
        .cli_config(CliConfig::default())
        .filename_match_threshold(1)
        .build();

    if let Err(e) = lib(&config) {
        panic!("There should have been no error. Found: {e}");
    }

    let report = get_report(PATHS.as_slice(), Some(config));

    assert_eq!(report.reports.len(), 2, "{:?}", report.reports);
}

/// [`foo.md`](./assets/logseq/pages/foo.md) and [`fooo.md`](./assets/logseq/pages/fooo.md) should conflict because
/// they are so similar. But we want to ignore it.
#[test]
fn test_ignore_word_pairs1() {
    info!("test_ignore_word_pairs");
    let config = config::Config::builder()
        .pages_directory(
            PathBuf::from_str("./tests/logseq/similar_filename/assets/pages")
                .expect("This is a constant"),
        )
        .file_config(FileConfig::default())
        .cli_config(CliConfig::default())
        .filename_match_threshold(1)
        .ignore_word_pairs(vec![("foo".to_string(), "fooo".to_string())])
        .build();

    if let Err(e) = lib(&config) {
        panic!("There should have been no error. Found: {e}");
    }

    let report = get_report(PATHS.as_slice(), Some(config));

    assert_eq!(report.reports.len(), 1, "{:?}", report.reports);
}

/// [`bar.md`](./assets/logseq/pages/bar.md) and [`barr.md`](./assets/logseq/pages/barr.md) should conflict because
/// they are so similar. But we want to ignore it.
#[test]
fn test_ignore_word_pairs2() {
    info!("test_ignore_word_pairs");
    let config = config::Config::builder()
        .pages_directory(
            PathBuf::from_str("./tests/logseq/similar_filename/assets/pages")
                .expect("This is a constant"),
        )
        .file_config(FileConfig::default())
        .cli_config(CliConfig::default())
        .filename_match_threshold(1)
        .ignore_word_pairs(vec![
            ("bar".to_string(), "barr".to_string()),
            ("foo".to_string(), "fooo".to_string()),
        ])
        .build();

    if let Err(e) = lib(&config) {
        panic!("There should have been no error. Found: {e}");
    }

    let report = get_report(PATHS.as_slice(), Some(config));

    assert_eq!(report.reports.len(), 0, "{:?}", report.reports);
}

#[test]
fn test_logseq_same_group() {
    let spacing = Regex::new("-|_|\\s").expect("Constant");

    // (file1, file2, expected_result)
    let cases = vec![
        ("fooo", "foo___bar", false),
        ("fooo", "foo", false),
        ("foo___bar", "fooo", false),
        ("foo___bar", "foo", true),
        ("foo___bar", "barr", false),
        ("foo", "fooo", false),
        ("foo", "foo___bar", true),
        ("foo", "barr", false),
        ("barr", "fooo", false),
        ("barr", "foo___bar", false),
        ("barr", "foo", false),
    ];

    for (f1, f2, expected) in cases {
        let path1 = PathBuf::from(f1);
        let path2 = PathBuf::from(f2);
        let result = SimilarFilename::skip_special_cases(&path1, &path2, &spacing)
            .expect("These are all constants");
        assert_eq!(
            result, expected,
            "Failure on pair {f1:?} : {f2:?}, expected {expected:?}, got {result:?}"
        );
    }
}
