mod logseq;
mod yaml;

use super::{wikilink::Alias, Error};

#[derive(Debug, Default, Clone)]
pub struct FrontMatter {
    /// The aliases of the file
    pub aliases: Vec<Alias>,
}

impl FrontMatter {
    pub(super) fn new(contents: &str) -> Result<Self, Error> {
        // Try to parse as YAML
        let out = yaml::Config::new(contents)?;
        if !out.is_empty() {
            return Ok(FrontMatter {
                aliases: out.aliases.iter().map(|x| Alias::new(x)).collect(),
            });
        }

        // Try to parse as Logseq
        let out = logseq::Config::new(contents)?;
        if !out.is_empty() {
            return Ok(FrontMatter {
                aliases: out.aliases.iter().map(|x| Alias::new(x)).collect(),
            });
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
                Alias::new("name1"),
                Alias::new("name2"),
                Alias::new("name3")
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
            vec![Alias::new("a"), Alias::new("b"), Alias::new("c")]
        );
    }
}
