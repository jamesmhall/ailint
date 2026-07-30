# Contributing to ailint

Thanks for your interest in improving **ailint**.

The full engineering conventions live in [AGENTS.md](AGENTS.md). This
file is the human-facing summary — it covers how to get set up, open a
PR, and where to look for deeper docs.

## Ways to contribute

- File a bug or feature request in
  [GitHub issues](https://github.com/jamesmhall/ailint/issues).
- Add or improve a lint rule (see [Adding a rule](#adding-a-rule)).
- Add support for a new AI-agent guidance file format.
- Improve reporter output (terminal, JSON, SARIF, Markdown).
- Improve docs — including this file, [README.md](README.md), or the
  [AGENTS.md](AGENTS.md) conventions.

## Prerequisites

- Rust toolchain, **1.75** or newer (MSRV).
- A recent `cargo`. No other system dependencies for the core CLI.

## Getting the code

```bash
git clone https://github.com/jamesmhall/ailint.git
cd ailint
cargo build --workspace
cargo test  --workspace
```

## Workspace layout

- `crates/ailint-core` — discovery, parsing, rule engine, reporters.
  Dependency-light. No network calls.
- `crates/ailint-llm` — optional AI-graded analyzer and provider
  clients. All network / LLM code lives here.
- `crates/ailint-cli` — the `ailint` binary.

## Before you open a PR

Run every check locally. All five must pass:

```bash
cargo build --workspace
cargo test  --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo run -p ailint-cli -- check .
```

The last command runs ailint against its own guidance files —
dogfooding. See the full PR checklist in
[AGENTS.md](AGENTS.md#pr-checklist).

## Adding a rule

Rules follow the `AILNNN` numeric-code + kebab-case slug pattern
(e.g. `AIL101` / `no-vague-instruction`). Ranges are documented in
[README.md](README.md#rules).

Full step-by-step is in
[AGENTS.md](AGENTS.md#adding-a-lint-rule). In short:

1. Pick the next unused code in the right category range.
2. Add the rule struct under `crates/ailint-core/src/rules/{category}/`.
3. Register it in
   [crates/ailint-core/src/rules/registry.rs](crates/ailint-core/src/rules/registry.rs).
4. Add a positive fixture, a negative fixture, and an integration test.

## Adding a supported file type

File-type detection lives in
[crates/ailint-core/src/file_type.rs](crates/ailint-core/src/file_type.rs).
Add the `FileType` variant, a detection branch, an `as_str()` arm, and
a unit test. Do not broaden detection with speculative patterns — only
add a branch when there's a real, publicly documented tool convention
to match.

## Code style

- No `.unwrap()` / `.expect()` in `ailint-core` or `ailint-llm` outside
  `#[cfg(test)]`.
- No `reqwest` or provider SDK imports in `ailint-core`. All network
  code lives in `ailint-llm`.
- User-visible output goes through a `ReporterKind`, never `println!`.
- Follow `cargo fmt --all`; do not hand-format.
- Fix clippy findings instead of `#[allow(...)]`-ing them; if you must
  allow, leave a one-line justification.

## Security

- Never commit real API keys, tokens, or `.env` files.
- Fixture data must not look like a real secret. Use obvious
  placeholders (`sk-test-EXAMPLE`, `xxxx…`).
- If you find a security bug in ailint itself, please open a private
  advisory on GitHub rather than a public issue.

## Commit and PR conventions

- Keep each commit focused. Prefer a small stack over one large diff.
- Reference the issue number in the PR description when applicable.
- Fill in the PR checklist (from
  [AGENTS.md](AGENTS.md#pr-checklist)) before requesting review.

## License

By contributing, you agree that your contributions will be licensed
under the same dual license as the project:
[MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).
