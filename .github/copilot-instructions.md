# GitHub Copilot instructions

Repo-wide guidance for GitHub Copilot on **ailint**. This file is loaded
as context on every chat and completion request. Full conventions live
in [AGENTS.md](../AGENTS.md).

## What ailint is

A Rust workspace (`ailint-core`, `ailint-llm`, `ailint-cli`) that lints
AI agent guidance files. MSRV: Rust 1.75, edition 2021.

## Rules for generated code

- Use `anyhow::Result` at binary boundaries, `thiserror` for library
  error enums. No `.unwrap()` / `.expect()` in `ailint-core` or
  `ailint-llm` except in tests.
- No network calls in `ailint-core`. Every `reqwest` / provider SDK
  import belongs in `ailint-llm`.
- New lint rules are registered in
  [crates/ailint-core/src/rules/registry.rs](../crates/ailint-core/src/rules/registry.rs)
  and follow the `AILNNN` numeric-code + kebab-case slug pattern.
- New file-type detections go in
  [crates/ailint-core/src/file_type.rs](../crates/ailint-core/src/file_type.rs).
- User-visible output goes through a `ReporterKind`, not `println!`.

## Build and validation

```bash
cargo build --workspace
cargo test  --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo run -p ailint-cli -- check .
```

All five must pass before a PR is opened.

## What not to suggest

- Do not suggest `.unwrap()` in library code.
- Do not suggest adding `reqwest` / `openai` / `anthropic` dependencies
  to `ailint-core`.
- Do not suggest bypassing `cargo fmt` or clippy with `#[allow]` without
  a one-line justification.
- Do not suggest committing API keys, `.env` files, or fixture files
  containing real secrets.
