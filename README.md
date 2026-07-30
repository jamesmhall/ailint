<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="media/logo_vertical_dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="media/logo_vertical.svg">
    <img src="media/logo_vertical.svg" alt="ailint logo" width="300" />
  </picture>
</div>

<br />

**Lint and inspect AI agent guidance files.**

`ailint` is an open-source CLI that scans a repository for AI agent guidance —
`CLAUDE.md`, `AGENTS.md` (also used by Google Antigravity, OpenAI Codex, and
Aider), GitHub Copilot instructions, VS Code `.instructions.md` /
`.prompt.md` / `.agent.md`, Cursor / Windsurf / Cline rules, JetBrains Junie
guidelines, and generic system prompts. It reports structural, semantic,
security, and cross-file consistency issues.

## The Missing Guardrail for Agentic Engineering

When scaling agentic engineering, unlinted guidance files lead to catastrophic context drift. Without strict enforcement, AI agents and copilots can run amuck—executing dangerous commands, hallucinating implementations, and making destructive edits to critical business logic.

`ailint` acts as the essential firewall. It ensures your agents remain strictly aligned with team policies and safely within their operational lanes, guaranteeing that vital human-in-the-loop oversight is augmented by rigid, automated rule enforcement.

## Status

- Structural, semantic, security, and consistency rules — 18 rules total
  (15 per-doc + 3 cross-file batch).
- 4 reporters: colored terminal, JSON, SARIF 2.1.0 (GitHub code scanning),
  and Markdown.
- Optional LLM analyzer (`AIL900`) with OpenAI, Anthropic, Google, and
  Ollama providers; opt-in via `--llm-provider`.
- `.ailint.yaml` config discovery walks parent directories from the target.
- Slim CLI build supported via `cargo build -p ailint-cli --no-default-features`
  (drops the `ailint-llm` crate + tokio).
- Packaging: cross-compiled release binaries, Docker image, `npx` wrapper,
  GitHub Action, and pre-commit hook.

## Features

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

# Docker
docker run --rm -v "$PWD":/src ghcr.io/jamesmhall/ailint check /src

# pre-commit hook
# repos:
#   - repo: https://github.com/jamesmhall/ailint
#     rev: v1.0.1
#     hooks:
#       - id: ailint

# Homebrew (in-repo tap)
brew tap jamesmhall/ailint https://github.com/jamesmhall/ailint
brew install jamesmhall/ailint/ailint
```

## Usage

```bash
ailint check .                       # lint current directory
ailint check . --format sarif -o out.sarif
ailint stats .                       # coverage / rule-density report
ailint init                          # scaffold .ailint.yaml
ailint list-rules                    # print all rules
ailint schema                        # JSON Schema for .ailint.yaml
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
Either can be used to enable / disable / suppress. Full documentation for
every rule lives in [docs/rules/README.md](docs/rules/README.md).

| Range | Category | Examples |
|-------|----------|----------|
| AIL001–099 | Structural | invalid frontmatter, empty file, missing section |
| AIL100–199 | Semantic | vague instruction, missing example, duplicate rule |
| AIL200–299 | Security | prompt injection marker, unrestricted tool grant, secrets |
| AIL300–399 | Consistency | conflicting rules across files |
| AIL900–999 | LLM (opt-in) | AI-graded clarity score |

Run `ailint list-rules` for the current full list.

## Workspace layout

The repository is structured to separate the core engine, LLM integrations, and the CLI binary:

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

## About the Project

`ailint` is an open-source initiative built out of the practical necessities of production-grade AI orchestration. It was created by [Jamie Hall](https://linkedin.com/in/jamesmhall), an industry software architect with 25 years of software development experience specializing in high-level AI agent architecture and complex system design.

After directing engineering teams and architecting cross-platform strategies that autonomously convert Jira tickets into mergeable pull requests, it became clear that the biggest bottleneck to scaling AI isn't the models—it's managing context drift. `ailint` was built in the true spirit of open-source software to enforce rigid architectural guidelines, eliminate "AI slop," and provide the essential tooling necessary for reliable, standardized code generation.

Contributions, discussions, and collaborations are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup and PR flow, and
[AGENTS.md](AGENTS.md) for the full engineering conventions.

## License

Dual-licensed under **MIT** ([LICENSE-MIT](LICENSE-MIT)) or **Apache-2.0**
([LICENSE-APACHE](LICENSE-APACHE)) at your option.
