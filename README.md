# ailint

**Lint and inspect AI agent guidance files.**

`ailint` is an open-source CLI that scans a repository for AI agent guidance —
`CLAUDE.md`, `AGENTS.md`, GitHub Copilot instructions, VS Code `.instructions.md`
/ `.prompt.md` / `.agent.md`, Cursor / Windsurf / Cline rules, and generic
system prompts — and reports structural, semantic, security, and cross-file
consistency issues.

> Status: **early scaffold**. Everything is stubbed; see `TODO` markers.

## Features (planned)

- Auto-detects AI agent guidance files across a project tree
- Rule categories:
  - **Structural** — schema / frontmatter validation
  - **Semantic** — vague instructions, missing examples, duplicated rules
  - **Security** — prompt injection markers, dangerous permissions, secrets
  - **Consistency** — conflicting rules across multiple guidance files
  - **LLM (opt-in)** — AI-graded quality analysis via OpenAI / Anthropic /
    Google / Ollama / any OpenAI-compatible endpoint
- Output formats: colored terminal, JSON, SARIF (GitHub code scanning), Markdown
- CI-first: non-zero exit on violations, GitHub Action included
- Configurable via `.ailint.yaml`

## Install

```bash
# Rust toolchain
cargo install ailint-cli

# npx wrapper (downloads a prebuilt binary)
npx ailint check .

# Homebrew (TODO: publish tap)
# brew install ailint

# Docker
# docker run --rm -v "$PWD":/src ghcr.io/OWNER/ailint check /src
```

## Usage

```bash
ailint check .                       # lint current directory
ailint check . --format sarif -o out.sarif
ailint stats .                       # coverage / rule-density report
ailint init                          # scaffold .ailint.yaml
ailint list-rules                    # print all rules
```

## Configuration

`.ailint.yaml` in the project root:

```yaml
rules:
  disabled: []                       # rule IDs or slugs, e.g. [AIL100, no-vague-instruction]
  severity_overrides: {}             # e.g. { AIL005: warning }

paths:
  exclude: [node_modules, .git, dist, target]

llm:
  provider: openai                   # openai | anthropic | google | ollama
  model: gpt-4o
  # API key comes from AILINT_LLM_API_KEY or the provider-specific env var.

output:
  format: terminal                   # terminal | json | sarif | markdown
  color: auto                        # auto | always | never
```

## Rules

Each rule has a numeric code (`AIL001`) and a slug (`no-vague-instruction`).
Either can be used to enable / disable / suppress.

| Range | Category | Examples |
|-------|----------|----------|
| AIL001–099 | Structural | invalid frontmatter, empty file, missing section |
| AIL100–199 | Semantic | vague instruction, missing example, duplicate rule |
| AIL200–299 | Security | prompt injection marker, unrestricted tool grant, secrets |
| AIL300–399 | Consistency | conflicting rules across files |
| AIL900–999 | LLM (opt-in) | AI-graded clarity score |

Run `ailint list-rules` for the current full list.

## Workspace layout

```
ailint/
├── crates/
│   ├── ailint-core/   # discovery, parsing, rule engine, reporters
│   ├── ailint-llm/    # optional LLM analyzer + provider integrations
│   └── ailint-cli/    # the `ailint` binary
├── action.yml         # GitHub Action
├── npm/               # npx wrapper
└── docker/Dockerfile
```

## License

Dual-licensed under **MIT** ([LICENSE-MIT](LICENSE-MIT)) or **Apache-2.0**
([LICENSE-APACHE](LICENSE-APACHE)) at your option.
