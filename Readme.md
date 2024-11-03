# MdLinker

A linter whose goal is to lint wikilinks in a variety of markdown note taking apps to enable maximal networked thinking.

# Lint Rules

- [X] Similar Files: Two files share a very similar title. Maybe you should combine them?
- [X] Duplicate Alias: If using something like [logseq aliases](https://unofficial-logseq-docs.gitbook.io/unofficial-logseq-docs/beginner-to-advance-features/aliases), make sure they are always unique.
- [X] Broken Wikilink: Some wikilink's derived resource does not exist. Maybe you should create it, or maybe the link title is misspelled?
- [ ] Misspelled File Name: Mispellings can make it hard for the linker to do its job.
- [ ] Misspelled Alias: Mispellings can make it hard for the linker to do its job.
- [ ] Unlinked Text: Text was detected which is very similar to some file title or alias. Maybe you should wrap it as a link?
- [ ] Missing "Related To":
  - [ ] If one block contains 2 or more wikilinks, add each of them to each other's "Related To"
  - [ ] If one block with a wikilink has a subblock with a wikilink, add each of them to each other's "Related To"

# Compatibility

- [X] Logseq Folder Structure
- [X] Yaml Front Matter
- [X] Logseq Aliases (in Yaml Front Matter)
- [X] `[[title]]` Wikilinks
- [X] #[[title]] tags
- [ ] #title tags
- [ ] Obsidian Folder Structure
- [ ] Obsidian Aliases
