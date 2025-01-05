use lazy_static::lazy_static;
use mdlinker::rules::broken_wikilink;

use crate::common::get_report;
use log::{debug, info};
use mdlinker::rules::filter_code;

use itertools::Itertools;

lazy_static! {
    static ref PATHS: Vec<String> = vec![
        "./tests/logseq/broken_wikilink/assets/pages/".to_string(),
        "./tests/logseq/broken_wikilink/assets/journals/".to_string()
    ];
}

#[test]
fn number_of_broken_wikilinks() {
    info!("number_of_broken_wikilinks");
    let report = get_report(PATHS.as_slice(), None);
    for broken_wikilink in &report.broken_wikilinks() {
        debug!("{broken_wikilink:?}");
    }
    assert_eq!(report.broken_wikilinks().len(), 5);
}

/// This passes because the link is valid
#[test]
fn lorem_exist_and_is_wikilink() {
    info!("lorem_exist_and_is_wikilink");
    let report = get_report(PATHS.as_slice(), None);
    for broken_wikilink in &report.broken_wikilinks() {
        debug!("{broken_wikilink:?}");
    }
    assert!(filter_code(
        report.broken_wikilinks(),
        &format!("{}::2024_11_01::lorem", broken_wikilink::CODE).into()
    )
    .is_empty());
}

/// This fails because the link is invalid
#[test]
fn ipsum_does_not_exist_and_is_wikilink() {
    info!("ipsum_does_not_exist_and_is_wikilink");
    let report = get_report(PATHS.as_slice(), None);
    for broken_wikilink in &report.broken_wikilinks() {
        debug!("{broken_wikilink:?}");
    }
    assert!(!filter_code(
        report.broken_wikilinks(),
        &format!("{}::2024_11_01::ipsum", broken_wikilink::CODE).into()
    )
    .is_empty());
}

/// This passes because there is no link
#[test]
fn dolor_does_not_exist_and_is_not_wikilink_in_journal() {
    info!("dolor_does_not_exist_and_is_not_wikilink_in_journal");
    let report = get_report(PATHS.as_slice(), None);
    for broken_wikilink in &report.broken_wikilinks() {
        debug!("{broken_wikilink:#?}");
    }
    assert!(filter_code(
        report.broken_wikilinks(),
        &format!("{}::2024_11_01::dolor", broken_wikilink::CODE).into()
    )
    .is_empty());
}

/// This passes because the link is valid
/// This is also a tag so testing that tags work
#[test]
fn sit_exists_and_is_tag() {
    info!("sit_exists_and_is_tag");
    let report = get_report(PATHS.as_slice(), None);
    for broken_wikilink in &report.broken_wikilinks() {
        debug!("{broken_wikilink:#?}");
    }
    assert!(filter_code(
        report.broken_wikilinks(),
        &format!("{}::2024_11_01::sit", broken_wikilink::CODE).into()
    )
    .is_empty());
}

/// This fails because the link is invalid
/// This is a multi word tag, just testing those work
#[test]
fn amet_does_not_exist_and_is_fancy_tag() {
    info!("amet_does_not_exist_and_is_fancy_tag");
    let report = get_report(PATHS.as_slice(), None);
    for broken_wikilink in &report.broken_wikilinks() {
        debug!("{broken_wikilink:#?}");
    }
    assert!(!filter_code(
        report.broken_wikilinks(),
        &format!("{}::2024_11_01::amet", broken_wikilink::CODE).into()
    )
    .is_empty());
}

/// This fails because the link is invalid
/// This is a multi word tag, just testing those work
#[test]
fn consectetur_does_not_exist_and_is_tag() {
    info!("consectetur_does_not_exist_and_is_tag");
    let report = get_report(PATHS.as_slice(), None);
    for broken_wikilink in &report.broken_wikilinks() {
        debug!("{broken_wikilink:#?}");
    }
    assert!(!filter_code(
        report.broken_wikilinks(),
        &format!("{}::2024_11_01::consectetur", broken_wikilink::CODE).into()
    )
    .is_empty());
}

/// This fails because the link is invalid
/// This is a regular tag gut on another line, just testing .decendants work
#[test]
fn adipiscing_does_not_exist_and_is_tag() {
    info!("adipiscing_does_not_exist_and_is_tag");
    let report = get_report(PATHS.as_slice(), None);
    for broken_wikilink in &report.broken_wikilinks() {
        debug!("{broken_wikilink:#?}");
    }
    assert!(!filter_code(
        report.broken_wikilinks(),
        &format!("{}::2024_11_01::adipiscing", broken_wikilink::CODE).into()
    )
    .is_empty());
}

/// This fails because the link is invalid
/// This is a regular tag gut on another line, just testing .decendants work
#[test]
fn elit_exists_and_is_tag() {
    info!("elit_exists_and_is_tag");
    let report = get_report(PATHS.as_slice(), None);
    for broken_wikilink in &report.broken_wikilinks() {
        debug!("{broken_wikilink:#?}");
    }
    assert!(filter_code(
        report.broken_wikilinks(),
        &format!("{}::2024_11_01::elit", broken_wikilink::CODE).into()
    )
    .is_empty());
}

#[test]
fn dolor_does_not_exist_and_is_wikilink_in_foo() {
    info!("dolor_does_not_exist_and_is_not_wikilink_in_foo");
    let report = get_report(PATHS.as_slice(), None);
    for broken_wikilink in &report.broken_wikilinks() {
        debug!("{broken_wikilink:#?}");
    }
    assert!(!filter_code(
        report.broken_wikilinks(),
        &format!("{}::foo::dolor", broken_wikilink::CODE).into()
    )
    .is_empty());
}

#[test]
fn dolor_does_not_exist_and_is_wikilink_in_foo_span() {
    info!("dolor_does_not_exist_and_is_not_wikilink_in_foo");
    let report = get_report(PATHS.as_slice(), None);
    for broken_wikilink in &report.broken_wikilinks() {
        debug!("{broken_wikilink:#?}");
    }
    let err_list = filter_code(
        report.broken_wikilinks(),
        &format!("{}::foo::dolor", broken_wikilink::CODE).into(),
    );
    let err = err_list.iter().exactly_one().unwrap();
    assert_eq!(err.wikilink.offset(), 62);
    assert_eq!(err.wikilink.len(), 5);
}
