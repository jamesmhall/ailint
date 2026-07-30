---
applyTo: "**/*.rs"
description: "Rust conventions for ailint."
---

# Rust conventions

Apply these rules whenever editing or generating Rust in this repo. Full
context is in [AGENTS.md](../../AGENTS.md).

## Error handling

- Return `anyhow::Result<T>` from binary code (`ailint-cli`) and from
  functions that fan out into many possible errors.
- Define library-level errors with `thiserror`.
- Never `.unwrap()` or `.expect()` in library code outside `#[cfg(test)]`
  modules.

## Style

- Follow `cargo fmt --all`. No hand-formatted alignment.
- Fix `cargo clippy -- -D warnings`. Only `#[allow(...)]` with a
  one-line reason comment on the same or previous line.
- Prefer `Vec::new()` / `Default::default()` over allocating throwaway
  placeholders.
- Use `Path` / `PathBuf` — not raw `String` — for filesystem paths.

## Structure

- `ailint-core` is dependency-light. No `reqwest`, no provider SDKs, no
  async runtime imports.
- `ailint-llm` owns all network and LLM code.
- New public items in `ailint-core` need a doc comment describing what
  a caller should know — not restating the signature.

## Tests

- Every new rule ships with a positive fixture (the rule fires) and a
  negative fixture (the rule stays silent).
- Unit tests for `file_type::detect` go in a `#[cfg(test)]` module in
  the same file.
