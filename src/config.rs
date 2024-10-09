mod cli;
mod default;
mod file;
use std::path::PathBuf;

use crate::sed::ReplacePair;
use clap::Parser;
use miette::{miette, Result};

pub struct Config {
    cli: cli::Cli,
    file: file::Config,
}

impl Config {
    pub fn new() -> Result<Self> {
        let cli = cli::Cli::parse();

        // If the config file doesn't exist, and it's not the default, error out
        let mut file = file::Config::default();
        if cli.config_path.is_file() {
            file = file::Config::new(&cli.config_path)
                .expect("Config file should be valid after these checks");
        } else if cli.config_path != PathBuf::from("mdlinker.toml") {
            return Err(miette!("Config file does not exist: {:?}", cli.config_path));
        }

        Ok(Self { cli, file })
    }

    pub fn directories(&self) -> Vec<PathBuf> {
        let mut out = self.cli.directories.clone();
        out.extend(self.file.directories.clone());
        if out.is_empty() {
            default::directories()
        } else {
            out
        }
    }

    pub fn ngram_size(&self) -> usize {
        self.cli
            .ngram_size
            .unwrap_or_else(|| self.file.ngram_size.unwrap_or_else(default::ngram_size))
    }

    pub fn boundary_pattern(&self) -> String {
        self.cli.boundary_pattern.clone().unwrap_or_else(|| {
            self.file
                .boundary_pattern
                .clone()
                .unwrap_or_else(default::boundary_pattern)
        })
    }

    pub fn filename_spacing_pattern(&self) -> String {
        self.cli
            .filename_spacing_pattern
            .clone()
            .unwrap_or_else(|| {
                self.file
                    .filename_spacing_pattern
                    .clone()
                    .unwrap_or_else(default::filename_spacing_pattern)
            })
    }

    pub fn filename_match_threshold(&self) -> i64 {
        self.cli.filename_match_threshold.unwrap_or_else(|| {
            self.file
                .filename_match_threshold
                .unwrap_or_else(default::filename_match_threshold)
        })
    }

    pub fn exclude(&self) -> Vec<String> {
        let mut out = self.cli.exclude.clone();
        out.extend(self.file.exclude.clone());
        if out.is_empty() {
            default::exclude()
        } else {
            out
        }
    }

    pub fn filepath_to_title(&self) -> Result<Vec<Vec<ReplacePair>>> {
        let mut out: Vec<Vec<ReplacePair>> = vec![];
        for outer in self.file.filepath_to_title.clone() {
            for inner in outer {
                let mut temp: Vec<ReplacePair> = vec![];
                let (from, to) = inner;
                temp.push(ReplacePair::new(&from, &to)?);
                out.push(temp);
            }
        }
        if out.is_empty() {
            out = default::filepath_to_title();
        }
        Ok(out)
    }

    pub fn title_to_filepath(&self) -> Result<Vec<Vec<ReplacePair>>> {
        let mut out: Vec<Vec<ReplacePair>> = vec![];
        for outer in self.file.title_to_filepath.clone() {
            for inner in outer {
                let mut temp: Vec<ReplacePair> = vec![];
                let (from, to) = inner;
                temp.push(ReplacePair::new(&from, &to)?);
                out.push(temp);
            }
        }
        if out.is_empty() {
            out = default::title_to_filepath();
        }
        Ok(out)
    }
}
