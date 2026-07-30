# Project guidelines (AGENTS)

The canonical Rust workspace layout for ailint is three crates:
`ailint-core`, `ailint-llm`, and `ailint-cli`. Keep network calls out of
`ailint-core` so it stays offline-friendly. Register every new rule in
`registry::all_rules` and add a fixture pair.
