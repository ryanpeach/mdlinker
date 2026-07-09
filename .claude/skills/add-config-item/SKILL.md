---
name: add-config-item
description: >-
  Add a new configuration setting to mdlinker (a config option / config item /
  new CLI flag). Use when asked to add, expose, or wire up a new setting that
  should be readable from the TOML config file and/or the command line, with
  CLI values taking priority over file values.
---

# Add a config item to mdlinker

Config lives in `src/config/`. Three types cooperate:

- `src/config/file.rs` — `Config` (the **partial**): every field is `Option<T>`,
  `#[derive(... Merge)]`. It is both the TOML representation and the merge target.
- `src/config/cli.rs` — `Config` (clap `Parser`) plus `to_partial()`, which maps CLI
  args into the partial.
- `src/config.rs` — the master `Config` (final, resolved, non-`Option` fields with
  `bon` `#[builder(default = ...)]`) and `resolve()`, the single place defaults,
  transforms, and validation happen. `merge` gives CLI priority: in `resolve`,
  `cli.to_partial()` then `.merge(file)` keeps the CLI's `Some` and only fills its
  `None` holes from the file.

## Steps

### 1. Add the field to the partial — `src/config/file.rs`
Add an `Option<T>` field with `#[serde(default)]`. Use a raw/string-y type if the
value needs transformation or validation later (globs, regex pairs, error codes):

```rust
/// See [`super::Config::my_setting`]
#[serde(default)]
pub my_setting: Option<usize>,
```

No `#[merge(...)]` attribute is needed — `merge`'s built-in `Merge for Option`
already keeps the left `Some` and fills `None` (overwrite-none), which is exactly the
CLI-priority behaviour.

### 2. (CLI-exposed only) Add the clap arg and map it — `src/config/cli.rs`
Add the clap field:

```rust
/// What my setting does
#[clap(long = "my-setting")]
pub my_setting: Option<usize>,
```

Then add one line to `to_partial()`:

```rust
my_setting: self.my_setting,
```

Conventions to preserve: empty multi-value/positional args map to `None` (so an unset
CLI value does not clobber the file); boolean flags map to `Some(self.flag)` (a
CLI-set flag wins and the file cannot override it). Pick a **unique** clap short
letter — duplicate `short = '…'` makes clap panic on startup.

### 3. Add the resolved field + default and wire the merge — `src/config.rs`
On the master `Config` struct, add the final field with its default:

```rust
/// What my setting does
#[builder(default = 3)]
pub my_setting: usize,
```

In `resolve()`, add one line feeding the merged partial into the builder:

```rust
.maybe_my_setting(merged.my_setting)
```

If the value needs transformation (map to a domain type) or validation, do it here —
this is the one place for it. See `expand_files`, `build_replace_pair`, and the
`allow_dirty`/`allow_staged` mutual-exclusion check for existing patterns. For a
fallible transform, return `Result` and use `?` (there is a `NewConfigError` variant
to add to if you need a new error).

### 4. Consume it
Read it via `config.my_setting` wherever needed (e.g. `src/lib.rs`, `src/rules/*.rs`).

### 5. Verify
```
cargo clippy --all-targets    # repo denies pedantic warnings in CI style
cargo test
```
Add a small unit test in `src/config.rs`'s `mod tests` if the setting has a
transform or non-trivial merge/validation (see the existing tests there — construct a
`file::Config { .. }` literal and call `resolve`).

## Worked examples

**A file-only `usize` (2 edits):** add `Option<usize>` to `file.rs` partial;
add the resolved field + `#[builder(default)]` and one `.maybe_x()` line in
`config.rs`. Done — no `cli.rs` change.

**A CLI+file `bool` flag (4 edits):** `Option<bool>` in `file.rs`; clap `bool` field
in `cli.rs` + `my_flag: Some(self.my_flag)` in `to_partial`; resolved `bool` field
with `#[builder(default = false)]` + `.maybe_my_flag(merged.my_flag)` in `resolve`.

That's the whole surface — there is no trait to implement and no per-field merge glue.
