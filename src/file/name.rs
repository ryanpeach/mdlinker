use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use indicatif::ProgressBar;
use miette::{Diagnostic, NamedSource, SourceOffset, SourceSpan};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

use crate::error::HasId;
use crate::ngrams;

/// Walk the directories and get just the files
fn get_files(dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for path in dirs {
        let walk = WalkDir::new(path);
        for entry in walk.into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                out.push(entry.into_path());
            }
        }
    }
    out
}

/// Get the filename from a path
/// Does not include the file extension
fn get_filename(path: &Path) -> String {
    let fname = path
        .file_name()
        .expect("We were given a guaranteed file path, not a directory")
        .to_string_lossy();
    return fname
        .split('.')
        .next()
        .expect("File paths will either have a file extension or not, it makes no difference")
        .to_string();
}

/// Generate n-grams from the filenames found in the directories
pub fn ngrams(
    dirs: Vec<PathBuf>,
    ngram_size: usize,
    boundary_regex: &Regex,
    filename_spacing_regex: &Regex,
) -> HashMap<String, PathBuf> {
    let files = get_files(dirs);
    let mut file_name_ngrams = HashMap::new();
    for filepath in files {
        let filename = get_filename(&filepath);
        let ngrams = ngrams::up_to_n(
            &filename,
            ngram_size,
            boundary_regex,
            filename_spacing_regex,
        );
        log::debug!("Filename: {}, ngrams: {:?}", filename, ngrams.len());
        file_name_ngrams.insert(filename, filepath);
    }
    file_name_ngrams
}

const CODE: &str = "file::name::similar";

#[derive(Error, Debug, Diagnostic)]
#[error("Filenames are similar")]
#[diagnostic(code(CODE))]
pub struct SimilarFilenames {
    /// Used to identify the diagnostic and exclude it if needed
    id: String,

    #[source_code]
    filepaths: NamedSource<String>,

    #[label("This bit here")]
    file1_ngram: SourceSpan,

    #[label("That bit there")]
    file2_ngram: SourceSpan,

    #[help]
    advice: String,
}

impl SimilarFilenames {
    /// Create a new `SimilarFilenames` diagnostic
    /// based on the two filenames and their similar ngrams
    pub fn new(
        file1_path: &Path,
        file1_ngram: &str,
        file2_path: &Path,
        file2_ngram: &str,
        score: i64,
    ) -> Self {
        // file paths as strings
        let file1 = file1_path.to_string_lossy();
        let file2 = file2_path.to_string_lossy();

        // Assemble the source
        let source = format!("{file1}\n{file2}");
        let filepaths = NamedSource::new("Filepaths", source.clone());

        // Find the ngrams in each filepath
        let find1 = file1
            .find(file1_ngram)
            .expect("Parameters say that this is a substring of file1");
        let find2 = file2
            .find(file2_ngram)
            .expect("Parameters say that this is a substring of file2");

        // Create the spans
        let file1_ngram = SourceSpan::new(
            SourceOffset::from_location(&source, 1, find1 + 1),
            file1_ngram.len(),
        );
        let file2_ngram = SourceSpan::new(
            SourceOffset::from_location(&source, 2, find2 + 1),
            file2_ngram.len(),
        );

        // filenames sorted
        let filename1 = get_filename(file1_path);
        let filename2 = get_filename(file2_path);
        let (filename1, filename2) = if filename1 < filename2 {
            (filename1, filename2)
        } else {
            (filename2, filename1)
        };

        // Create the unique id
        let id = format!("{CODE}::{filename1}::{filename2}");

        // Create the advice
        let advice = format!(
            "Maybe you should combine them into a single file?\nThe score was: {score:?}\nid: {id:?}"
        );
        Self {
            id,
            filepaths,
            file1_ngram,
            file2_ngram,
            advice,
        }
    }

    pub fn calculate(
        file_ngrams: &std::collections::HashMap<String, PathBuf>,
        filename_match_threshold: i64,
    ) -> Vec<SimilarFilenames> {
        // Convert all filenames to a single string
        // Check if any two file ngrams fuzzy match
        // TODO: Unfortunately this is O(n^2)
        #[allow(clippy::cast_precision_loss)]
        let n = file_ngrams.len() as f64;
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let file_crosscheck_bar = ProgressBar::new((n * (n + 1.0) / 2.0) as u64);
        let matcher = SkimMatcherV2::default();
        let mut matches: Vec<SimilarFilenames> = Vec::new();
        for (i, (ngram, filepath)) in file_ngrams.clone().iter().enumerate() {
            // We can start at i + 1 because we've already checked all previous files
            for (other_ngram, other_filepath) in file_ngrams.iter().skip(i + 1) {
                file_crosscheck_bar.inc(1);

                // Skip if the same file
                if filepath == other_filepath {
                    continue;
                }

                // Each editor will have its own special cases, lets centralize them
                if SimilarFilenames::skip_special_cases(filepath, other_filepath) {
                    continue;
                }

                // Score the ngrams and check if they match
                let score = matcher.fuzzy_match(ngram, other_ngram);
                if let Some(score) = score {
                    if score > filename_match_threshold {
                        log::info!("Match! {:?} and {:?}", filepath, other_filepath);
                        log::debug!(
                            "Ngrams: '{}' and '{}', Score: {:?}",
                            ngram,
                            other_ngram,
                            score
                        );
                        matches.push(SimilarFilenames::new(
                            filepath,
                            ngram,
                            other_filepath,
                            other_ngram,
                            score,
                        ));
                        break;
                    }
                } else {
                    log::debug!("No match: {} and {}", ngram, other_ngram);
                }
            }
        }
        file_crosscheck_bar.finish();
        matches
    }
}

/// Each editor will have its own special cases, lets centralize them
impl SimilarFilenames {
    /// Centralize the special cases for skipping
    fn skip_special_cases(file1: &Path, file2: &Path) -> bool {
        SimilarFilenames::logseq_same_group(file1, file2)
    }

    /// Logseq has a special case if one startswith the other then
    /// its probably a part of the same group
    fn logseq_same_group(file1: &Path, file2: &Path) -> bool {
        let file1 = get_filename(file1);
        let file2 = get_filename(file2);
        file1.starts_with(&file2) || file2.starts_with(&file1)
    }
}

impl HasId for SimilarFilenames {
    fn id(&self) -> String {
        self.id.clone()
    }
}
