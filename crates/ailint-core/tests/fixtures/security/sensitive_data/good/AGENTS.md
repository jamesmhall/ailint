# Good sensitive-data fixture

Documentation strings that resemble API keys are permitted so long as they
are clearly marked as placeholders. AIL202 must not fire on any of these:

- `sk-test-EXAMPLE-xxxx` (short placeholder token)
- `sk-EXAMPLEabcdefghijklmnopqrstuvwxyz` (EXAMPLE marker in context)
- `sk-placeholderabcdefghijklmnopqrstuvwxyz`

Refer to `AILINT_LLM_API_KEY` for the real value at runtime.
