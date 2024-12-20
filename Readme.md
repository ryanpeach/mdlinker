# MdLinker

A linter whose goal is to lint wikilinks in a variety of markdown note taking apps to enable maximal networked thinking.

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

Options are defined in [`src/config/file.rs`](src/config/file.rs) as a serde object, and can be overwritten in the cli, see `mdlinker --help`.

# Lint Rules

- [X] Similar Files: Two files share a very similar title. Maybe you should combine them! Uses a fuzzy finder and ngram similarity. O(n^2) complexity in the number of files.
- [X] Duplicate Alias: If using something like [logseq aliases](https://unofficial-logseq-docs.gitbook.io/unofficial-logseq-docs/beginner-to-advance-features/aliases), make sure they are always unique (also compares them to filenames).
- [X] Broken Wikilink: Some wikilinks linked resource does not exist. Maybe you should create the page, or maybe the link title is misspelled?
- [X] Unlinked Text: Text was detected which is very similar to some file title or alias. Maybe you should wrap it as a link?
- [ ] RelatesTo: At least in logseq, I find it annoying the graph wont show two items as linked if they are linked within the same block. These will identify that and force you to link them in the frontmatter.
  - RelatesToParagraph: Two wikilinks in the same paragraph
  - RelatesToListItem: Two wikilinks in the same list item
  - RelatesToListItemDescendant: One wikilink in a sublist item of another wikilinks listitem
  - RelatesToNotFound: delete a relation which doesnt match any of the above for cleanliness

# Future

- [ ] LSP Implementation
- [ ] Logseq Plugin

# Compatibility

- [X] Logseq Folder Structure
- [X] Logseq Hierarchy
- [X] Yaml Front Matter
- [X] Logseq Aliases (in Yaml Front Matter)
- [X] `[[url]]` and `[[title|url]]` style wikilinks
- [X] #[[url]] and #url tags
- [ ] Marksman [[#url]] tags
- [ ] Logseq properties ":: style" (Won't implement, use yaml front matter)
- [ ] Obsidian Folder Structure (Unknown, PRs welcome)
- [ ] Obsidian Aliases (Unknown, PRs welcome)
- [ ] [Marksman](https://github.com/artempyanykh/marksman)
