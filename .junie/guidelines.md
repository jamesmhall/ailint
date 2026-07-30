# Junie guidelines — ailint

Project-scoped guidelines for JetBrains Junie. Full conventions live
in [AGENTS.md](../AGENTS.md).

## Project

Rust workspace: `ailint-core`, `ailint-llm`, `ailint-cli`. MSRV Rust
1.75, edition 2021.

## Rules

- Use `anyhow::Result` at binary boundaries; use `thiserror` for library
  error enums.
- No `.unwrap()` / `.expect()` in `ailint-core` or `ailint-llm` outside
  `#[cfg(test)]`.
- No network calls in `ailint-core`. Every `reqwest` or provider SDK
  import belongs in `ailint-llm`.
- New lint rules are registered in
  `crates/ailint-core/src/rules/registry.rs::all_rules` and follow the
  `AILNNN` numeric-code + kebab-case slug pattern.
- New file-type detections go in `crates/ailint-core/src/file_type.rs`
  with a matching `as_str()` arm and a unit test.
- User-visible output goes through a `ReporterKind`, never `println!`.
- Every new rule ships with a positive and a negative fixture.
- Do not commit real API keys or `.env` files.

## Build and validate

```
cargo build --workspace
cargo test  --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo run -p ailint-cli -- check .
```

All five must pass before Junie opens a PR.

## Junie-specific notes

- Prefer running `cargo` commands through Junie's terminal integration
  rather than shelling out for each check.
- When planning multi-step changes, put the plan under `.junie/plans/`
  (Junie's convention) and keep this `guidelines.md` focused on rules.
