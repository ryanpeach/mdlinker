use crate::sed::ReplacePair;
use std::path::PathBuf;

pub fn directories() -> Vec<PathBuf> {
    vec![PathBuf::from(".")]
}

pub fn ngram_size() -> usize {
    2
}

pub fn boundary_pattern() -> String {
    r"[,./_]".to_string()
}

pub fn filename_spacing_pattern() -> String {
    r"___|__|-|_|\s".to_string()
}

pub fn filename_match_threshold() -> i64 {
    0
}

pub fn exclude() -> Vec<String> {
    vec![]
}

pub fn title_to_filepath() -> Vec<Vec<ReplacePair>> {
    vec![vec![
        ReplacePair::new(r"\[\[(.*?)\]\]", r"$1.md").expect("Constant"),
        ReplacePair::new(r"/", r"___").expect("Constant"),
        ReplacePair::new(r"(.*)", r"../pages/$1").expect("Constant"),
    ]]
}

pub fn filepath_to_title() -> Vec<Vec<ReplacePair>> {
    vec![vec![
        ReplacePair::new(r"([A-Za-z0-1_-]+).md", r"\[\[$1\]\]").expect("Constant"),
        ReplacePair::new(r"___", r"/").expect("Constant"),
    ]]
}
