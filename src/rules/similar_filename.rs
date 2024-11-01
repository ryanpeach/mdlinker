use crate::{file::name::get_filename, rules::HasId, sed::MissingSubstringError};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use indicatif::ProgressBar;
use miette::{Diagnostic, SourceOffset, SourceSpan};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const CODE: &str = "name::similar";

#[derive(Error, Debug, Diagnostic)]
#[error("Filenames are similar")]
#[diagnostic(code(CODE))]
pub struct SimilarFilename {
    /// Used to identify the diagnostic and exclude it if needed
    id: String,

    #[source_code]
    filepaths: String,

    #[label("This bit here")]
    file1_ngram: SourceSpan,

    #[label("That bit there")]
    file2_ngram: SourceSpan,

    #[help]
    advice: String,
}

impl PartialEq for SimilarFilename {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl HasId for SimilarFilename {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl SimilarFilename {
    /// Create a new diagnostic
    /// based on the two filenames and their similar ngrams
    ///
    pub fn new(
        file1_path: &Path,
        file1_ngram: &str,
        file2_path: &Path,
        file2_ngram: &str,
        score: i64,
    ) -> Result<Self, MissingSubstringError> {
        // file paths as strings
        let file1 = file1_path.to_string_lossy().to_lowercase();
        let file2 = file2_path.to_string_lossy().to_lowercase();

        // Assemble the source
        let source = format!("{file1}\n{file2}");
        let filepaths = source.clone();

        // Find the ngrams in each filepath
        let find1 = file1.find(file1_ngram).ok_or_else(|| {
            MissingSubstringError::builder()
                .path(file1_path.to_path_buf())
                .ngram(file1_ngram.to_string())
                .build()
        })?;
        let find2 = file2.find(file2_ngram).ok_or_else(|| {
            MissingSubstringError::builder()
                .path(file2_path.to_path_buf())
                .ngram(file2_ngram.to_string())
                .build()
        })?;

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
        Ok(Self {
            id,
            filepaths,
            file1_ngram,
            file2_ngram,
            advice,
        })
    }

    pub fn calculate(
        file_ngrams: &std::collections::HashMap<String, PathBuf>,
        filename_match_threshold: i64,
    ) -> Result<Vec<SimilarFilename>, MissingSubstringError> {
        // Convert all filenames to a single string
        // Check if any two file ngrams fuzzy match
        // TODO: Unfortunately this is O(n^2)
        #[allow(clippy::cast_precision_loss)]
        let n = file_ngrams.len() as f64;
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let file_crosscheck_bar = ProgressBar::new((n * (n + 1.0) / 2.0) as u64);
        let matcher = SkimMatcherV2::default();
        let mut matches: Vec<SimilarFilename> = Vec::new();
        for (i, (ngram, filepath)) in file_ngrams.clone().iter().enumerate() {
            // We can start at i + 1 because we've already checked all previous files
            for (other_ngram, other_filepath) in file_ngrams.iter().skip(i + 1) {
                file_crosscheck_bar.inc(1);

                // Skip if the same file
                if filepath == other_filepath {
                    continue;
                }

                // Each editor will have its own special cases, lets centralize them
                if SimilarFilename::skip_special_cases(filepath, other_filepath) {
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
                        matches.push(SimilarFilename::new(
                            filepath,
                            ngram,
                            other_filepath,
                            other_ngram,
                            score,
                        )?);
                        break;
                    }
                } else {
                    log::debug!("No match: {} and {}", ngram, other_ngram);
                }
            }
        }
        file_crosscheck_bar.finish();
        Ok(matches)
    }
}

/// Each editor will have its own special cases, lets centralize them
impl SimilarFilename {
    /// Centralize the special cases for skipping
    fn skip_special_cases(file1: &Path, file2: &Path) -> bool {
        SimilarFilename::logseq_same_group(file1, file2)
    }

    /// Logseq has a special case if one startswith the other then
    /// its probably a part of the same group
    fn logseq_same_group(file1: &Path, file2: &Path) -> bool {
        let file1 = get_filename(file1);
        let file2 = get_filename(file2);
        file1.starts_with(&file2) || file2.starts_with(&file1)
    }
}
