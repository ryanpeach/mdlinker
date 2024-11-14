use std::fs;

use lazy_static::lazy_static;
use mdlinker::rules::unlinked_text;

use log::{debug, info};
use mdlinker::rules::filter_code;
use miette::SourceOffset;

use crate::common::get_report;

use itertools::Itertools;

lazy_static! {
    static ref PATHS: Vec<String> = vec![
        "./tests/logseq/unlinked_text/assets/pages/".to_string(),
        "./tests/logseq/unlinked_text/assets/journals/".to_string()
    ];
}

#[test]
fn number_of_unlinked_texts() {
    info!("number_of_unlinked_texts");
    let report = get_report(PATHS.as_slice());
    for unlinked_texts in &report.unlinked_texts() {
        debug!("{unlinked_texts:#?}");
    }
    assert_eq!(report.unlinked_texts().len(), 3);
}

/// This passes because the link is valid
#[test]
fn lorem_exist_and_is_wikilink() {
    info!("lorem_exist_and_is_wikilink");
    let report = get_report(PATHS.as_slice());
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
    let report = get_report(PATHS.as_slice());
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
    let report = get_report(PATHS.as_slice());
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
    let report = get_report(PATHS.as_slice());
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
    let report = get_report(PATHS.as_slice());
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
