---
applyTo: "**/*.md"
description: "Style rules for AI-agent guidance files in this repo."
---

# Guidance-file style

Apply these when editing any Markdown file in this repo, especially
agent guidance files like [AGENTS.md](../../AGENTS.md), [CLAUDE.md](../../CLAUDE.md),
`.cursorrules`, `.windsurfrules`, `.clinerules`, and files under
`.cursor/rules/`, `.github/instructions/`, `.junie/`, and `.claude/rules/`.

## Voice

- Use imperatives ("Register the rule in …") over vague passives
  ("Rules should generally be registered somewhere").
- Prefer concrete examples over abstract description. If you say "add a
  fixture," show what the fixture looks like or link to one.
- Keep sentences short. Aim for one instruction per bullet.

## Structure

- Start with a one-line summary of what the file governs.
- Follow with a section that names the source of truth. In this repo,
  that source of truth is [AGENTS.md](../../AGENTS.md).
- Group related rules under `##` headings. Don't nest deeper than `###`
  unless the file is long enough to warrant a table of contents.

## Consistency

- Build / test / lint commands must match [AGENTS.md](../../AGENTS.md)
  exactly. Copy-paste them; don't paraphrase.
- MSRV and edition are stated identically across all files: Rust 1.75,
  edition 2021.
- Use relative Markdown links to files in this repo, not bare filenames.

## What not to do

- Don't restate the same rule in different words across multiple files.
  Point at [AGENTS.md](../../AGENTS.md) instead.
- Don't use tone words like "please" or "kindly" — agents don't need
  politeness, they need precision.
- Don't invent tool conventions. If a tool has no publicly documented
  file format, don't fabricate one.
