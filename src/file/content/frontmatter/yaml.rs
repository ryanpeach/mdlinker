//! ---
//! name: value
//! ---

use serde::{Deserialize, Serialize};

use miette::{miette, Result};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    /// The aliases of the file
    #[serde(default)]
    pub alias: Vec<String>,
}

impl Config {
    pub fn new(contents: &str) -> Result<Self> {
        // See if contents contains "---" and a newline and another "---" using multiline regex
        let re = regex::Regex::new(r"(?s)---\n(.*)\n---").expect("Its a constant.");
        let frontmatter = re.captures(contents);

        // If we don't find the frontmatter, return the default
        match frontmatter {
            None => Ok(Self::default()),
            Some(caps) => {
                // Parse the YAML
                serde_yaml::from_str(&caps[1]).map_err(|e| miette!(e))
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.alias.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let text = "---\nalias: [\"a\",\"b\",\"c\"]\n---";
        // create the config
        let config = Config::new(text).unwrap();
        assert_eq!(
            config.alias,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }
}
