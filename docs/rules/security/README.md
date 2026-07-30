# Security Rules (AIL200 - AIL299)

Security rules provide static analysis of guidance files to prevent the accidental embedding of credentials and to stop dangerous AI instructions that might lead to remote code execution or data loss.

| Code | Slug | Description |
|------|------|-------------|
| [AIL200](AIL200.md) | `no-prompt-injection-marker` | Detects phrases like "Ignore previous instructions", acting as a safeguard. |
| [AIL201](AIL201.md) | `no-unrestricted-tool-grant` | Flags when instructions give the agent uninhibited permission to run commands. |
| [AIL202](AIL202.md) | `no-sensitive-data-in-instructions` | Detects when API keys or secrets are embedded straight into rule files instead of using environments or parameter passing. |
| [AIL203](AIL203.md) | `tool-confirmation-required` | Requires human-confirmation phrasing when instructions describe destructive actions. |
