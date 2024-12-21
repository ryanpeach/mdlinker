# MdLinker

A linter whose goal is to lint wikilinks in a variety of markdown note taking apps to enable maximal networked thinking. Currently supports [logseq](https://logseq.com/). Is fundamentally designed to aide in the [zettelkasten method](https://zettelkasten.de/overview/) of note taking.

Uses [miette](https://github.com/zkat/miette) for beautiful :crab: rust style error messages. :crab:

Uses git [pre-commit](https://pre-commit.com/) to integrate with your git workflow.

```yaml
- repo: https://github.com/ryanpeach/mdlinker
  rev: <VERSION>
  hooks:
    - id: mdlinker
```

Linking works best when you spell things correctly, in both your filenames and file contents. Also this tool only works with ASCII characters at the moment (see https://github.com/ryanpeach/mdlinker/issues/45). I recommend adding the following pre-commit hooks before this one:

```yaml
  - repo: https://github.com/crate-ci/typos
    rev: v1.23.7
    hooks:
      - id: typos
  - repo: https://github.com/ryanpeach/mdlinker
    rev: <VERSION>
    hooks:
      - id: enforce-ascii
      - id: mdlinker
```

# Configuration

Put a `mdlinker.toml` in your project root to configure the linter.

Options are defined in [`src/config/file.rs`](src/config/file.rs) as a serde object, and can be overwritten in the cli, see `mdlinker --help` and the docstrings for full details.

```toml
# This is the folder where filenames which represent linkable words go.
pages_directory = "pages"

# These are any other folders you wish to scan. These can't be linked to. Usually these are things which link to items in the pages_directory.
other_directories = ["journal", "notes"]

# Exclusions
# This is how you silence specific rules or instances of errors
# It accepts glob patterns
exclude = [
    "rule:category:*",
    "rule:category:error:id:as:found:in:the:error:output",
    "..."
]
ignore_word_pairs = [
    ["foo", "foobar"],
]  # These are pairs of words which look similar in your filenames but are not the same. Suppresses SimilarFilename rule.

# The Similar Filename rule can match on n_grams, like "Barrack Obama". But in order to do this, you need to set the max number of words in an ngram.
# You really don't need to change any of these
ngram_size = 3
boundary_pattern = r"___" # This is a regex pattern to match on filenames to stop ngram generation (like at a hierarchy or sentence boundary). In logseq this is represented with three underscores.
filename_match_threshold = 100 # This is the similarity threshold for the similar filename rule. It is an integer corresponding to the output of the [fuzzy-matcher](https://github.com/skim-rs/fuzzy-matcher) crate.
filename_spacing_pattern = "-|_|\s" # This is a regex pattern to split filenames into words. It is used for the ngram generation.

# Compatibility
# These are options that are meant to help us eventually prototype this system for other tools like obsidian. They convert filenames in the "pages_directory" to aliases, and aliases to filenames in the "pages_directory". Do not change these unless you know what you are doing.
filename_to_alias = ["___", "/"]
alias_to_filename = ["/", "___"]
```

# Lint Rules

- [X] Similar Files: Two files share a very similar title. Maybe you should combine them! Uses a fuzzy finder and ngram similarity. O(n^2) complexity in the number of files.
- [X] Duplicate Alias: If using something like [logseq aliases](https://unofficial-logseq-docs.gitbook.io/unofficial-logseq-docs/beginner-to-advance-features/aliases), make sure they are always unique (also compares them to filenames).
- [X] Broken Wikilink: Some wikilinks linked resource does not exist. Maybe you should create the page, or maybe the link title is misspelled?
- [X] Unlinked Text: Text was detected which is very similar to some file title or alias. Maybe you should wrap it as a link?

# Compatibility

- [X] Logseq Folder Structure
- [X] Logseq Hierarchy
- [X] Yaml Front Matter
- [X] Logseq Aliases (in Yaml Front Matter)
- [X] `[[url]]` and `[[title|url]]` style wikilinks
- [X] #[[url]] and #url tags
- [ ] Links to other files in the "other_directories"
- [ ] Marksman [[#url]] tags
- [ ] Logseq properties ":: style" (Won't implement, use yaml front matter)
- [ ] Obsidian Folder Structure (PRs welcome)
- [ ] Obsidian Aliases (PRs welcome)
- [ ] [Marksman](https://github.com/artempyanykh/marksman)
- [ ] [Roam](https://roamresearch.com/)
- [ ] [Zettelkasten](https://zettelkasten.de/)
