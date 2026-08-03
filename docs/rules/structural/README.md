# Structural Rules (AIL001 - AIL099)

Structural rules check the schema, frontmatter, and file-level health of your guidance files. If a file cannot be parsed or lacks the fundamental structure required by agent environments, these rules will report a violation.

| Code | Slug | Description |
|------|------|-------------|
| [AIL001](AIL001.md) | `no-frontmatter-schema-error` | Validates YAML frontmatter formatting against expected schemas. |
| [AIL002](AIL002.md) | `instructions-file-empty` | Detects zero-byte or whitespace-only files that provide no guidance. |
| [AIL003](AIL003.md) | `missing-required-section` | Identifies when files lack certain mandatory sections (e.g. `Examples`). |
| [AIL004](AIL004.md) | `mcp-schema-validation` | Validates the minimum schema of MCP server config files (`mcp.json`, `.cline_mcp.json`). |
