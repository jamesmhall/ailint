# Guidelines

- When writing a new lint rule for the project, first identify the numeric code range that applies to the rule category, then pick the next unused number in that range, then write both a positive and a negative fixture, then register the rule in the central registry, and finally add integration tests covering the fire and silence paths before opening the pull request.
