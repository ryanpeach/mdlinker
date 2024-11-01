mod logseq;
mod yaml;

use super::Error;

#[derive(Debug, Default, Clone)]
pub struct FrontMatter {
    /// The aliases of the file
    pub aliases: Vec<String>,
}

impl FrontMatter {
    pub(super) fn new(contents: &str) -> Result<Self, Error> {
        // Try to parse as YAML
        let out = yaml::Config::new(contents)?;
        if !out.is_empty() {
            return Ok(FrontMatter { aliases: out.alias });
        }

        // Try to parse as Logseq
        let out = logseq::Config::new(contents)?;
        if !out.is_empty() {
            return Ok(FrontMatter { aliases: out.alias });
        }

        // If we can't parse it, return the default
        Ok(Self::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_logseq() {
        let text = "\nalias:: name1,name2,name3\n";
        // create the config
        let config = FrontMatter::new(text).unwrap();
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
    fn test_new_yaml() {
        let text = "---\nalias: [\"a\",\"b\",\"c\"]\n---";
        // create the config
        let config = FrontMatter::new(text).unwrap();
        assert_eq!(
            config.aliases,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }
}
