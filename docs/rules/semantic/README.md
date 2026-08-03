# Semantic Rules (AIL100 - AIL199)

Semantic rules analyze the language, clarity, and usefulness of the instructions provided to AI agents. These rules help eliminate "AI slop" and ensure that agents receive distinct, actionable commands.

| Code | Slug | Description |
|------|------|-------------|
| [AIL100](AIL100.md) | `no-vague-instruction` | Flags instructions with non-committal language like "be helpful" or "if possible". |
| [AIL101](AIL101.md) | `no-missing-examples` | Flags sections called "Examples" which don't actually provide concrete source code snippets or cases. |
| [AIL102](AIL102.md) | `excessive-rule-length` | Identifies single list-item rules that exceed recommended word counts, making them hard for LLMs to digest. |
| [AIL103](AIL103.md) | `no-duplicate-rules` | Detects when the same instruction essentially repeats itself inside a single file. |
| [AIL104](AIL104.md) | `negative-constraint-overload` | Flags files where negative constraints ("do not", "never") dominate affirmative guidance. |
| [AIL105](AIL105.md) | `vendor-optimization-syntax` | Recommends XML tags (like `<conventions>`) in guidance files for Anthropic-hosted agents. |
| [AIL106](AIL106.md) | `detect-instruction-bloat` | Flags monolithic prose paragraphs that agents are prone to skim. |
