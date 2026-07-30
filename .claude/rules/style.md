# Style rule (Claude)

Style guidance for Claude Code when editing this repo. Full conventions
live in [AGENTS.md](../../AGENTS.md).

## Comment style

- Write a comment only to state what the code cannot show on its own.
- Keep single-line comments to one line. No multi-paragraph explainer
  blocks in front of a one-liner.
- Do not restate what the next line does in prose.
- Do not narrate a change ("Fixed the bug where…") in code comments;
  put that in the commit message instead.

## Naming

- Rule slugs are kebab-case (`no-vague-instruction`), not snake_case.
- `RuleId` constants use `RuleId::new(<code>, "<slug>")` with a numeric
  code in the right `AILNNN` range.
- Module directories under `crates/ailint-core/src/rules/` match the
  category name exactly: `structural`, `semantic`, `security`,
  `consistency`.

## Diff hygiene

- Do not reformat unrelated code in the same PR as a functional change.
- Do not add `TODO:` comments without a concrete follow-up.
