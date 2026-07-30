# Consistency Rules (AIL300 - AIL399)

Consistency rules act as batch-processing rules. Unlike single-document rules, cross-file rules examine the full corpus of instruction files across your workspace to find instances where instructions conflict, contradict, or uselessly duplicate one another.

| Code | Slug | Description |
|------|------|-------------|
| [AIL300](AIL300.md) | `no-conflicting-rules` | Flags rules across files that seem to state direct opposites of one another. |
| [AIL301](AIL301.md) | `no-duplicate-guidance-files` | Warns when entire guidance files are identical. |
