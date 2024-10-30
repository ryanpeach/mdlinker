# MdLinker

A linter whose goal is to lint wikilinks in a variety of markdown note taking apps to enable maximal networked thinking.

# Lint Rules

- [X] Similar Files: Two files share a very similar title. Maybe you should combine them?
- [ ] Broken Wikilink: Some wikilink's derived file path does not exist. Maybe you should create it, or maybe the link title is misspelled?
- [ ] Text is not a Wikilink: Some text on a page is identical to some page name, perhaps it should be a wikilink?
- [ ] Misspelled File Name: Mispellings can make it hard for the linker to do its job.
- [ ] Misspelled Wikilink: Mispellings can make it hard for the linker to do its job.
- [ ] Unlinked Text: Text was detected which is very similar to some file title or alias. Maybe you should wrap it as a link?
- [ ] Missing "Related To":
  - [ ] If one block contains 2 or more wikilinks, add each of them to each other's "Related To"
  - [ ] If one block with a wikilink has a subblock with a wikilink, add each of them to each other's "Related To"

# Compatibility

- [X] Logseq
- [ ] Obsidian
- [ ] Roam
- [ ] Joplin
