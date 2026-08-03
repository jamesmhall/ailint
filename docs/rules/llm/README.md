# LLM Rules (AIL900 - AIL999)

LLM rules are an opt-in set of rules. Using your provided `.ailint.yaml` provider credentials, these rules hit actual intelligent models to perform complex rule validations that basic grep or syntax parsers miss.

| Code | Slug | Description |
|------|------|-------------|
| [AIL900](AIL900.md) | `llm-quality-score` | Sends the guidance file to the LLM to get a critique based on vagueness, lack of examples, contradictions, and security pitfalls. |
| [AIL901](AIL901.md) | `llm-actionability-check` | Asks the LLM to flag every directive an agent could not execute deterministically without asking follow-up questions. |
