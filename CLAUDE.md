# CLAUDE.md

Guidance for Claude Code working on **ailint**.

See [AGENTS.md](AGENTS.md) for the full conventions. This file only
adds Claude-specific tips.

## Before you touch code

- Read [AGENTS.md](AGENTS.md) once per session.
- Before adding a lint rule, read
  [crates/ailint-core/src/rules/mod.rs](crates/ailint-core/src/rules/mod.rs)
  and [crates/ailint-core/src/rules/registry.rs](crates/ailint-core/src/rules/registry.rs).
- Before adding a file-type variant, read
  [crates/ailint-core/src/file_type.rs](crates/ailint-core/src/file_type.rs).

## Working style

- Prefer `Read` on whole files (this repo is small) over many small
  ranged reads.
- Use the `Bash` tool for `cargo` commands. The canonical set is in
  [AGENTS.md](AGENTS.md#build-test-lint).
- Do not run `cargo publish`, `cargo yank`, or any `git push` command
  without explicit user confirmation.
- Do not modify `Cargo.lock` by hand; let `cargo` update it.
- Follow the [change workflow](AGENTS.md#change-workflow): feature
  branch → full validation → PR → green CI → squash merge. Never commit
  to `main`.
- Version bumps and tags follow
  [AGENTS.md](AGENTS.md#releases-and-versioning); the full release
  procedure is in [RELEASING.md](RELEASING.md). Every release is
  `scripts/release.sh X.Y.Z` — do not improvise.

## Constraints inherited from AGENTS.md

- No `.unwrap()` / `.expect()` in library code.
- `ailint-core` must not depend on `reqwest` or a provider SDK.
- Every new rule must be registered in `registry::all_rules` **and**
  have a positive + negative fixture.
- Every reporter output goes through `ReporterKind`, never ad-hoc
  `println!`.
