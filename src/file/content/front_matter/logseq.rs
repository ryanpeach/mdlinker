//! logseq uses:
//! <name>:: csv

use serde::{Deserialize, Serialize};

use regex::Regex;

use crate::file::Error;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    /// The aliases of the file
    #[serde(default)]
    pub aliases: Vec<String>,
}

fn parse_csv(contents: &str) -> Result<Vec<String>, Error> {
    contents
        .split(',')
        .map(|s| Ok(s.trim().to_string()))
        .collect()
}

impl Config {
    pub fn new(contents: &str) -> Result<Self, Error> {
        // find alias:: and capture the rest of the line as csv
        let re = Regex::new(r"alias::\s*(.*)").expect("Its a constant.");

        // parse the CSV
        let alias = re.captures(contents);

        match alias {
            None => Ok(Self::default()),
            Some(caps) => {
                // The first capture group is the regex match as a whole
                // The second is the parenthesized subexpression
                if caps.len() > 2 {
                    return Err(Error::DuplicateProperty("alias".to_owned()));
                }
                let alias =
                    parse_csv(&caps[1]).expect("Already checked for exactly one capture group.");
                Ok(Self { aliases: alias })
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
    fn test_parse_csv() {
        let contents = "a,b,c";
        let out = parse_csv(contents).unwrap();
        assert_eq!(out, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_new() {
        let text = "\nalias:: name1,name2,name3\n";
        // create the config
        let config = Config::new(text).unwrap();
        assert_eq!(
            config.aliases,
            vec![
                "name1".to_string(),
                "name2".to_string(),
                "name3".to_string()
            ]
        );
    }

    #[test]
    fn test_new_with_spaces() {
        let text = "alias:: name1, name2, name3";
        // create the config
        let config = Config::new(text).unwrap();
        assert_eq!(
            config.aliases,
            vec![
                "name1".to_string(),
                "name2".to_string(),
                "name3".to_string()
            ]
        );
    }
}
