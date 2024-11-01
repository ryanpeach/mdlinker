//! yaml uses:
//! ---
//! name: value
//! ---

use serde::{Deserialize, Serialize};

use crate::file::Error;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    /// The aliases of the file
    #[serde(default, rename = "alias")]
    pub aliases: Vec<String>,
}

impl Config {
    pub fn new(contents: &str) -> Result<Self, Error> {
        // See if contents contains "---" and a newline and another "---" using multiline regex
        let re = regex::Regex::new(r"(?s)---\n(.*)\n---").expect("Its a constant.");
        let frontmatter = re.captures(contents);

        // If we don't find the frontmatter, return the default
        match frontmatter {
            None => Ok(Self::default()),
            Some(caps) => {
                // Parse the YAML
                serde_yaml::from_str(&caps[1]).map_err(Error::SerdeError)
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty()
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
            config.aliases,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }
}
