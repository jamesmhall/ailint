# add-rule skill

Package the "add a new lint rule to ailint" workflow.

## Description

Scaffold a new lint rule end-to-end: allocate the next unused `AILNNN`
code in the chosen category, generate the rule struct, wire it into the
registry, and add positive + negative fixtures with an integration test.

Use this skill when a user asks to add, create, or scaffold a new
ailint rule, or refers to a rule by its intended slug (e.g. "add a
`no-vague-instruction` rule").

## When to use

- The user asks for a new lint rule and names either a category or a
  behavior.
- The user pastes a bad guidance snippet and asks "can ailint catch
  this?" — the answer is usually a new rule.

## When not to use

- Modifying an existing rule (edit the rule file directly).
- Adding a new file-type detection (that's `file_type.rs`, not a rule).
- Reporter or CLI changes (unrelated to rule authoring).

## Steps

1. **Confirm inputs.** Category (structural / semantic / security /
   consistency / LLM), one-sentence description, kebab-case slug.
2. **Read context.** Open
   [AGENTS.md](../../../AGENTS.md#adding-a-lint-rule),
   [crates/ailint-core/src/rules/mod.rs](../../../crates/ailint-core/src/rules/mod.rs),
   and
   [crates/ailint-core/src/rules/registry.rs](../../../crates/ailint-core/src/rules/registry.rs).
3. **Pick the code.** Scan the category directory for existing
   `RuleId::new(<code>, …)` values; pick the smallest unused code in
   the range from [README.md](../../../README.md#rules).
4. **Create the rule.** Add `crates/ailint-core/src/rules/{category}/{slug}.rs`
   implementing the `Rule` trait.
5. **Register.** Append a `Box::new(<Struct>::default())` line inside
   `all_rules()`.
6. **Fixtures + test.** Add positive and negative fixtures under
   `crates/ailint-core/tests/fixtures/{slug}/` and an integration
   test that asserts the exact violation set.
7. **Validate.** Run `cargo test --workspace`, `cargo clippy
   --workspace --all-targets -- -D warnings`, `cargo fmt --all`, and
   `cargo run -p ailint-cli -- check .`.

## Constraints

- Never reuse an `AILNNN` code.
- Never skip fixtures.
- Never add network dependencies to `ailint-core` (LLM rules go in
  `ailint-llm` behind the `AIL9NN` range).
