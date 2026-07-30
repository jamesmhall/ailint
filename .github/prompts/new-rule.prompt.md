---
mode: agent
description: "Scaffold a new ailint lint rule."
---

# New rule prompt

Use this prompt when adding a new lint rule to ailint. Follow every
step in order.

## Inputs to collect

Before generating code, ask the user for:

1. **Category** — structural, semantic, security, consistency, or LLM.
2. **What the rule checks** — one-sentence description in plain English.
3. **Suggested slug** — kebab-case, e.g. `no-vague-instruction`.

## Steps

1. Pick the next unused `AILNNN` code in the chosen category range. The
   ranges are defined in [AGENTS.md](../../AGENTS.md#adding-a-lint-rule)
   and [README.md](../../README.md#rules). Inspect
   [crates/ailint-core/src/rules/registry.rs](../../crates/ailint-core/src/rules/registry.rs)
   and the sibling module files to find the next free code.
2. Create the rule struct under
   `crates/ailint-core/src/rules/{category}/{slug}.rs`. Implement the
   `Rule` trait from
   [crates/ailint-core/src/rules/mod.rs](../../crates/ailint-core/src/rules/mod.rs).
   Return a `RuleId::new(<code>, "<slug>")` from `id()`.
3. Re-export the struct from the category `mod.rs`.
4. Register a boxed instance of the struct inside `all_rules()` in
   [crates/ailint-core/src/rules/registry.rs](../../crates/ailint-core/src/rules/registry.rs).
5. Add a positive fixture (rule fires) and a negative fixture (rule stays
   silent) under `crates/ailint-core/tests/fixtures/{slug}/`.
6. Add an integration test that runs the fixture through the rule and
   asserts the exact violation set.
7. Verify:
   - `cargo test --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo fmt --all`
   - `cargo run -p ailint-cli -- check .` still exits clean.

## Constraints

- Do not skip fixtures.
- Do not add `.unwrap()` or `.expect()` in the rule implementation.
- Do not put network or LLM code in a non-`AIL9NN` rule.
- Do not overlap `AILNNN` codes with existing rules — always check the
  registry first.
