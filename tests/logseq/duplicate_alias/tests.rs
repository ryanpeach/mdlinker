use mdlinker::rules::duplicate_alias;

use mdlinker::rules::duplicate_alias::DuplicateAlias;
use mdlinker::rules::filter_code;

use crate::common::get_report;
use glob::glob;
use log::{debug, info};
use std::path::PathBuf;

use itertools::Itertools;

static PATHS: std::sync::LazyLock<Vec<PathBuf>> = std::sync::LazyLock::new(|| {
    let first: Vec<PathBuf> = glob("./tests/logseq/duplicate_alias/assets/pages/**/*.md")
        .expect("This is a constant")
        .map(|p| p.expect("This is a constant"))
        .collect();
    let second: Vec<PathBuf> = glob("./tests/logseq/duplicate_alias/assets/journals/**/*.md")
        .expect("This is a constant")
        .map(|p| p.expect("This is a constant"))
        .collect();
    [first, second].concat()
});

#[test]
fn number_of_duplicate_alias() {
    info!("number_of_duplicate_alias");
    let report = get_report(PATHS.as_slice(), None);
    for duplicate_alias in &report.duplicate_aliases() {
        debug!("{duplicate_alias:#?}");
    }
    assert_eq!(report.duplicate_aliases().len(), 3);
}

#[test]
fn filename_alias_relation() {
    info!("filename_alias_relation");
    let report = get_report(PATHS.as_slice(), None);
    for duplicate_alias in &report.duplicate_aliases() {
        debug!("{duplicate_alias:#?}");
    }
    let duplicate = filter_code(
        report.duplicate_aliases(),
        &format!("{}::lorem", duplicate_alias::CODE).into(),
    )
    .into_iter()
    .at_most_one()
    .unwrap();
    assert!(duplicate.is_some());
}

#[test]
fn filecontent_filecontent_relation() {
    info!("filecontent_filecontent_relation");
    let report = get_report(PATHS.as_slice(), None);
    for duplicate_alias in &report.duplicate_aliases() {
        debug!("{duplicate_alias:#?}");
    }
    let duplicate = filter_code(
        report.duplicate_aliases(),
        &format!("{}::dolor", duplicate_alias::CODE).into(),
    )
    .into_iter()
    .at_most_one()
    .unwrap();
    assert!(duplicate.is_some());
}

#[test]
fn duplicate_ipsum() {
    info!("duplicate_ipsum_span");
    let report = get_report(PATHS.as_slice(), None);
    for duplicate_alias in &report.duplicate_aliases() {
        debug!("{duplicate_alias:#?}");
    }
    let duplicate = filter_code(
        report.duplicate_aliases(),
        &format!("{}::ipsum", duplicate_alias::CODE).into(),
    )
    .into_iter()
    .at_most_one()
    .unwrap();
    assert!(duplicate.is_some());
}

#[test]
fn duplicate_ipsum_span() {
    info!("duplicate_ipsum");
    let report = get_report(PATHS.as_slice(), None);
    let err_list = filter_code(
        report.duplicate_aliases(),
        &format!("{}::ipsum", duplicate_alias::CODE).into(),
    );
    let err = err_list.iter().exactly_one().unwrap();
    match err {
        DuplicateAlias::FileNameContentDuplicate { alias, .. }
        | DuplicateAlias::FileContentContentDuplicate { alias, .. } => {
            assert_eq!(alias.offset(), 11);
            assert_eq!(alias.len(), 5);
        }
    }
}
