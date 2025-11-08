use std::fs;

use mdlinker::rules::unlinked_text;

use glob::glob;
use log::{debug, info};
use mdlinker::rules::filter_code;
use miette::SourceOffset;
use std::path::PathBuf;

use crate::common::get_report;

use itertools::Itertools;

static PATHS: std::sync::LazyLock<Vec<PathBuf>> = std::sync::LazyLock::new(|| {
    let first: Vec<PathBuf> = glob("./tests/logseq/unlinked_text/assets/pages/**/*.md")
        .expect("This is a constant")
        .map(|p| p.expect("This is a constant"))
        .collect();
    let second: Vec<PathBuf> = glob("./tests/logseq/unlinked_text/assets/journals/**/*.md")
        .expect("This is a constant")
        .map(|p| p.expect("This is a constant"))
        .collect();
    [first, second].concat()
});

#[test]
fn number_of_unlinked_texts() {
    info!("number_of_unlinked_texts");
    let report = get_report(PATHS.as_slice(), None);
    for unlinked_texts in &report.unlinked_texts() {
        debug!("{unlinked_texts:#?}");
    }
    assert_eq!(report.unlinked_texts().len(), 4);
}

/// This passes because the link is valid
#[test]
fn lorem_exist_and_is_wikilink() {
    info!("lorem_exist_and_is_wikilink");
    let report = get_report(PATHS.as_slice(), None);
    for unlinked_text in &report.unlinked_texts() {
        debug!("{unlinked_text:#?}");
    }
    assert!(filter_code(
        report.unlinked_texts(),
        &format!("{}::2024_11_01::lorem", unlinked_text::CODE).into()
    )
    .is_empty());
}

/// This fails because the link is invalid
#[test]
fn ipsum_is_alias_and_is_not_wikilink_in_journal() {
    info!("ipsum_is_alias_and_is_not_wikilink_in_journal");
    let report = get_report(PATHS.as_slice(), None);
    for unlinked_text in &report.unlinked_texts() {
        debug!("{unlinked_text:#?}");
    }
    assert!(!filter_code(
        report.unlinked_texts(),
        &format!("{}::2024_11_01::ipsum", unlinked_text::CODE).into()
    )
    .is_empty());
}

#[test]
fn dolors_exists_and_is_not_wikilink_in_foo() {
    info!("dolors_exists_and_is_not_wikilink_in_foo");
    let report = get_report(PATHS.as_slice(), None);
    for unlinked_text in &report.unlinked_texts() {
        debug!("{unlinked_text:#?}");
    }
    assert!(!filter_code(
        report.unlinked_texts(),
        &format!("{}::foo::dolors", unlinked_text::CODE).into()
    )
    .is_empty());
}

#[test]
fn dolors_exists_and_is_not_wikilink_in_foo_span() {
    info!("dolors_exists_and_is_not_wikilink_in_foo_span");
    let report = get_report(PATHS.as_slice(), None);
    let err_list = filter_code(
        report.unlinked_texts(),
        &format!("{}::foo::dolors", unlinked_text::CODE).into(),
    );
    let err = err_list.iter().exactly_one().unwrap();
    assert_eq!(err.span.offset(), 62);
    assert_eq!(err.span.len(), 6);
}

/// This was not working on my notes, so I obscured it and added a test
#[test]
fn icazyvey_exists_and_is_not_wikilink_in_journal() {
    info!("icazyvey_exists_and_is_not_wikilink_in_journal");
    let report = get_report(PATHS.as_slice(), None);
    let err_list = filter_code(
        report.unlinked_texts(),
        &format!("{}::2024_08_10::icazyvey", unlinked_text::CODE).into(),
    );
    let err = err_list.iter().exactly_one().unwrap();
    let source = fs::read_to_string("./tests/logseq/unlinked_text/assets/journals/2024_08_10.md")
        .expect("This exists at compile time");
    let offset = SourceOffset::from_location(source, 11, 106);
    assert_eq!(err.span.offset(), offset.offset());
    assert_eq!(err.span.len(), 8);
}

/// Tests that linking is right after a non-standard character like "right parentheses" U+2019
#[test]
fn lorem_exists_and_is_not_wikilink_in_journal() {
    info!("lorem_exists_and_is_not_wikilink_in_journal");
    let report = get_report(PATHS.as_slice());
    let err_list = filter_code(
        report.unlinked_texts(),
        &format!("{}::foo::lorem", unlinked_text::CODE).into(),
    );
    let err = err_list.iter().exactly_one().unwrap();
    assert_eq!(err.span.offset(), 85);
    assert_eq!(err.span.len(), 5);
}
