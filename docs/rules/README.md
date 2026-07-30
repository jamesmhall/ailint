# ailint Rules Documentation

The `ailint` rule set is divided into categories, each covering a specific range of codes. This allows for organized enforcement and configuration of policies within your agentic development environment.

## Categories

| Range       | Category                                                    | Description                                                                 |
|-------------|-------------------------------------------------------------|-----------------------------------------------------------------------------|
| **AIL001–099** | [Structural](structural/README.md)                           | Ensures files are parseable, present, and possess correctly formed frontmatter. |
| **AIL100–199** | [Semantic](semantic/README.md)                               | Examines the meaning, clarity, and duplication of rules within individual files. |
| **AIL200–299** | [Security](security/README.md)                               | Highlights potential vulnerabilities like prompt injections and exposed secrets. |
| **AIL300–399** | [Consistency](consistency/README.md)                         | Identifies contradictions and duplications across multiple guidance files.       |
| **AIL900–999** | [LLM (opt-in)](llm/README.md)                                | Uses an active LLM connection to generate an AI-graded clarity and quality score. |

## Rule Configuration

You can enable, disable, and configure warnings directly in `.ailint.yaml`. Codes (e.g. `AIL001`) and their slugs (e.g. `no-frontmatter-schema-error`) can both be used inside your configuration file:

```yaml
rules:
  disabled:
    - AIL002
    - no-vague-instruction
  severity_overrides:
    AIL200: error
    AIL300: warning
```
