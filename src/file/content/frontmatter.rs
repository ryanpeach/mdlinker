mod logseq;
mod yaml;

use miette::Result;

#[derive(Debug, Default)]
pub struct Config {
    /// The aliases of the file
    pub alias: Vec<String>,
}

impl Config {
    pub fn new(contents: &str) -> Result<Self> {
        // Try to parse as YAML
        let out = yaml::Config::new(contents)?;
        if !out.is_empty() {
            return Ok(Config { alias: out.alias });
        }

        // Try to parse as Logseq
        let out = logseq::Config::new(contents)?;
        if !out.is_empty() {
            return Ok(Config { alias: out.alias });
        }

        // If we can't parse it, return the default
        Ok(Self::default())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_new_logseq() {
        let text = "\nalias:: name1,name2,name3\n";
        // create the config
        let config = Config::new(text).unwrap();
        assert_eq!(
            config.alias,
            vec![
                "name1".to_string(),
                "name2".to_string(),
                "name3".to_string()
            ]
        );
    }

    #[test]
    fn test_new_yaml() {
        let text = "---\nalias: [\"a\",\"b\",\"c\"]\n---";
        // create the config
        let config = Config::new(text).unwrap();
        assert_eq!(
            config.alias,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }
}
