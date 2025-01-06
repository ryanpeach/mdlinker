use crate::{
    config::{file::Config as FileConfig, Config},
    file::name::get_filename,
    ngrams::{CalculateError, Ngram},
};
use console::{style, Emoji};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use hashbrown::HashMap;
use indicatif::ProgressBar;
use miette::{Diagnostic, SourceOffset, SourceSpan};
use regex::Regex;
use std::backtrace::Backtrace;
use std::{
    env,
    path::{Path, PathBuf},
};
use thiserror::Error;

use super::{ErrorCode, FixError, IgnoreError, ReportTrait};

pub const CODE: &str = "name::similar";

static SIMILAR: Emoji<'_, '_> = Emoji("🤝  ", "");

#[derive(Error, Debug, Diagnostic, Clone)]
#[error("Filenames are similar")]
#[diagnostic(code("name::similar"))]
pub struct SimilarFilename {
    /// Used to identify the diagnostic and exclude it if needed
    id: ErrorCode,
    file1_ngram: Ngram,
    file2_ngram: Ngram,

    score: i64,

    #[source_code]
    filepaths: String,

    #[label("This bit here")]
    file1_ngram_span: SourceSpan,

    #[label("That bit there")]
    file2_ngram_span: SourceSpan,

    #[help]
    advice: String,
}
impl ReportTrait for SimilarFilename {
    fn id(&self) -> ErrorCode {
        self.id.clone()
    }
    fn fix(&self, _config: &Config) -> Result<Option<()>, FixError> {
        Ok(None)
    }
    fn ignore(&self, config: &mut FileConfig) -> Result<(), IgnoreError> {
        Ok(config
            .ignore_word_pairs
            .push((self.file1_ngram.to_string(), self.file2_ngram.to_string())))
    }
}

impl PartialOrd for SimilarFilename {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl PartialEq for SimilarFilename {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl SimilarFilename {
    /// Create a new diagnostic
    /// based on the two filenames and their similar ngrams
    ///
    pub fn new(
        file1_path: &Path,
        file1_ngram: &Ngram,
        file2_path: &Path,
        file2_ngram: &Ngram,
        spacing_regex: &Regex,
        score: i64,
    ) -> Result<Self, CalculateError> {
        // file paths as strings
        let file1 = file1_path.to_string_lossy().to_lowercase();
        let file2 = file2_path.to_string_lossy().to_lowercase();

        // Assemble the source
        let source = format!("{file1}\n{file2}");
        let filepaths = source.clone();

        // Find the ngrams in each filepath
        let find1 = spacing_regex
            .replace_all(&file1, " ")
            .find(&file1_ngram.to_string())
            .ok_or_else(|| CalculateError::MissingSubstringError {
                path: file1_path.to_path_buf(),
                ngram: file1_ngram.to_string(),
                backtrace: std::backtrace::Backtrace::capture(),
            })?;
        let find2 = spacing_regex
            .replace_all(&file2, " ")
            .find(&file2_ngram.to_string())
            .ok_or_else(|| CalculateError::MissingSubstringError {
                path: file2_path.to_path_buf(),
                ngram: file2_ngram.to_string(),
                backtrace: std::backtrace::Backtrace::capture(),
            })?;

        // Create the spans
        let file1_ngram_span = SourceSpan::new(
            SourceOffset::from_location(&source, 1, find1 + 1),
            file1_ngram.len(),
        );
        let file2_ngram_span = SourceSpan::new(
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
            id: id.into(),
            score,
            filepaths,
            file1_ngram_span,
            file2_ngram_span,
            advice,
            file1_ngram: file1_ngram.clone(),
            file2_ngram: file2_ngram.clone(),
        })
    }

    pub fn calculate(
        file_ngrams: &HashMap<Ngram, PathBuf>,
        filename_match_threshold: i64,
        spacing_regex: &Regex,
        config: &Config,
    ) -> Result<Vec<SimilarFilename>, CalculateError> {
        // Convert all filenames to a single string
        // Check if any two file ngrams fuzzy match
        // TODO: Unfortunately this is O(n^2)
        #[allow(clippy::cast_precision_loss)]
        let n = file_ngrams.len() as f64;
        let file_crosscheck_bar: Option<ProgressBar> = if env::var("RUNNING_TESTS").is_ok() {
            None
        } else {
            println!(
                "  {} {}Searching for Similar Filenames O(n^2)...",
                style("[1/3]").bold().dim(),
                SIMILAR
            );
            #[allow(clippy::cast_sign_loss)]
            #[allow(clippy::cast_possible_truncation)]
            Some(ProgressBar::new((n * (n + 1.0) / 2.0) as u64))
        };
        let matcher = SkimMatcherV2::default();
        let mut matches: Vec<SimilarFilename> = Vec::new();
        for (ngram, filepath) in file_ngrams {
            'outer: for (other_ngram, other_filepath) in file_ngrams {
                if ngram.nb_words() != other_ngram.nb_words() {
                    continue;
                }

                // TODO: This can be improved computationally using a hashmap
                // Skip based on ignore_word_pairs
                for (a, b) in &config.ignore_word_pairs {
                    println!("{a} || {b}");
                    if &ngram.to_string() == a && &other_ngram.to_string() == b {
                        continue 'outer;
                    }
                    if &ngram.to_string() == b && &other_ngram.to_string() == a {
                        continue 'outer;
                    }
                }

                if let Some(bar) = &file_crosscheck_bar {
                    bar.inc(1);
                }

                // Skip if the same file
                if filepath == other_filepath {
                    continue;
                }

                // Each editor will have its own special cases, lets centralize them
                if SimilarFilename::skip_special_cases(filepath, other_filepath, spacing_regex)? {
                    continue;
                }

                // Score the ngrams and check if they match
                let score = matcher.fuzzy_match(&ngram.to_string(), &other_ngram.to_string());
                if let Some(score) = score {
                    if score > filename_match_threshold {
                        matches.push(SimilarFilename::new(
                            filepath,
                            ngram,
                            other_filepath,
                            other_ngram,
                            spacing_regex,
                            score,
                        )?);
                        break;
                    }
                }
            }
        }
        if let Some(bar) = file_crosscheck_bar {
            bar.finish_and_clear();
        }
        Ok(matches)
    }
}

/// Each editor will have its own special cases, lets centralize them
impl SimilarFilename {
    pub fn skip_special_cases(
        file1: &Path,
        file2: &Path,
        spacing_regex: &Regex,
    ) -> Result<bool, CalculateError> {
        let file1_str = get_filename(file1).0;
        let file2_str = get_filename(file2).0;

        // If file1 is a prefix of file2 (with spacing), or file2 is a prefix of file1 (with spacing)
        // TODO: Compiling regex inside a loop is expensive
        let regex_str1 = format!("^{}({})", regex::escape(&file1_str), spacing_regex.as_str());
        let file1_is_prefix =
            Regex::new(&regex_str1).map_err(|e| CalculateError::RegexCompilationError {
                source: e,
                compilation_string: regex_str1,
                backtrace: Backtrace::force_capture(),
            })?;
        let regex_str2 = format!("^{}({})", regex::escape(&file2_str), spacing_regex.as_str());
        let file2_is_prefix =
            Regex::new(&regex_str2).map_err(|e| CalculateError::RegexCompilationError {
                source: e,
                compilation_string: regex_str2,
                backtrace: Backtrace::force_capture(),
            })?;

        let out1 = file1_is_prefix.is_match(&file2_str);
        let out2 = file2_is_prefix.is_match(&file1_str);
        let out = out1 || out2;
        println!(
            "({file1_str:?}, {file2_str:?}, {spacing_regex:?}) => ({out1} || {out2}) => {out}"
        );
        Ok(out)
    }
}
