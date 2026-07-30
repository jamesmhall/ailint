# Project guidelines

This project uses Rust 2021 edition with MSRV 1.75.

- Run `cargo fmt --all` before committing every change.
- Run `cargo clippy --workspace --all-targets` and treat warnings as errors.
- Add fixtures and integration tests for every new lint rule.
- Never commit real API keys or `.env` files to the repository.
