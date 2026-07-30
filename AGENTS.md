# AGENTS.md

Canonical guidance for AI coding agents working on **ailint**.

This file is the source of truth. `CLAUDE.md`, `.github/copilot-instructions.md`,
`.cursorrules`, `.windsurfrules`, `.clinerules`, `.junie/guidelines.md`, and the
files under `.cursor/rules/`, `.github/instructions/`, `.github/prompts/`,
`.github/agents/`, `.github/skills/`, and `.claude/rules/` all delegate here.
When conventions change, update this file first.

## Project overview

`ailint` is a Rust CLI that lints AI agent guidance files (`CLAUDE.md`,
`AGENTS.md`, GitHub Copilot instructions, Cursor / Windsurf / Cline / Junie
rules, generic system prompts). Full context is in [README.md](README.md).

Workspace layout:

- `crates/ailint-core` — discovery, parsing, rule engine, reporters. No
  network access, no LLM calls.
- `crates/ailint-llm` — optional AI-graded analyzer and provider clients.
  All network / LLM code lives here.
- `crates/ailint-cli` — the `ailint` binary. Argument parsing, config
  loading, dispatch.

MSRV: **Rust 1.75**, edition **2021**.

## Build, test, lint

Run these from the repo root:

```bash
cargo build --workspace
cargo test  --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

Self-lint (dogfood):

```bash
cargo run -p ailint-cli -- check .
```

## Coding conventions

- **Error handling.** Use `anyhow::Result` at binary boundaries and
  `thiserror` for library error enums. Do **not** use `.unwrap()` or
  `.expect()` in library code (`ailint-core`, `ailint-llm`) except in tests.
- **Panics.** No panics on user input. Parse errors surface as
  `Violation`s, not aborts.
- **Allocations.** Prefer `Vec::new()` / `Default::default()` over
  allocating placeholder values you'll immediately overwrite.
- **TODO markers.** OK to leave `TODO:` comments, but scope them to one
  concrete follow-up. Don't leave vague "improve this" notes.
- **Formatting.** `cargo fmt --all` is authoritative. Do not hand-format.
- **Lints.** `cargo clippy -- -D warnings` must pass. Fix clippy findings;
  don't `#[allow(...)]` them without a one-line reason comment.

## Adding a lint rule

1. Pick the next unused code in the right range (see [README.md](README.md)):
   - `AIL001–099` structural
   - `AIL100–199` semantic
   - `AIL200–299` security
   - `AIL300–399` consistency
   - `AIL900–999` LLM (opt-in)
2. Create the rule struct under
   `crates/ailint-core/src/rules/{category}/`. Implement the `Rule` trait
   from [crates/ailint-core/src/rules/mod.rs](crates/ailint-core/src/rules/mod.rs).
3. Give it a `RuleId` with a numeric code and a kebab-case slug (e.g.
   `RuleId::new(101, "no-vague-instruction")`).
4. Register it in
   [crates/ailint-core/src/rules/registry.rs](crates/ailint-core/src/rules/registry.rs)
   inside `all_rules()`.
5. Add a fixture under `crates/ailint-core/tests/fixtures/` and an
   integration test that asserts the rule fires (and does not fire on a
   negative fixture).
6. Cross-file rules go under `rules/consistency/` and implement the
   `BatchRule` trait from
   [crates/ailint-core/src/rules/mod.rs](crates/ailint-core/src/rules/mod.rs),
   registered in `registry::all_batch_rules`.

## Adding a supported file type

File-type detection lives in
[crates/ailint-core/src/file_type.rs](crates/ailint-core/src/file_type.rs).
Add a new `FileType` variant, a detection branch in `FileType::detect`,
an `as_str()` arm, and a unit test covering a representative path.

## LLM code

- Every network call goes through `crates/ailint-llm`. `ailint-core`
  must not depend on `reqwest` or any provider SDK.
- API keys come from environment variables (`AILINT_LLM_API_KEY` or
  provider-specific vars). **Never** commit real keys, `.env` files, or
  fixtures containing keys.
- LLM rules are opt-in (`AIL900–999`) and must degrade gracefully when
  no provider is configured.

## Output and reporters

Supported reporter kinds are `terminal`, `json`, `sarif`, `markdown`.
Adding a new reporter means a new variant on `ReporterKind` and a
matching match arm in the CLI's format mapping. Don't add ad-hoc print
statements — everything user-visible flows through a reporter.

## Configuration

`.ailint.yaml` at the repo root. Schema is defined in
[crates/ailint-core/src/config.rs](crates/ailint-core/src/config.rs).
When you add a config knob, update
[crates/ailint-cli/.ailint.yaml.template](crates/ailint-cli/.ailint.yaml.template)
in the same PR.

## PR checklist

Before opening a PR:

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` succeeds
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` is clean
- [ ] `cargo fmt --all --check` is clean
- [ ] `cargo run -p ailint-cli -- check .` runs without discovery errors
- [ ] New rules have both a positive and a negative fixture
- [ ] No new `.unwrap()` / `.expect()` in library code
- [ ] No committed secrets, API keys, or `.env` files

## What not to do

- Don't add network calls to `ailint-core`.
- Don't add rules with duplicated codes or slugs.
- Don't broaden `FileType::detect` with speculative patterns — add a
  detection branch only when there's a real tool convention to match.
- Don't silently swallow errors in the CLI. Surface them via `anyhow`
  and let the top level format them.
