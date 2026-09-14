---
paths:
  - "**/*.rs"
  - "**/*.sh"
  - "**/*.toml"
  - "**/*.yml"
  - "**/*.yaml"
---

# Code comments

- Do not add comments that merely restate what the code already makes obvious.
- Only add comments when they provide useful context that cannot be expressed clearly through the code itself, such as non-obvious constraints, reasoning, invariants, or workarounds.
- Remove any low-value or redundant comments introduced as part of your changes.

This rule is about comments in code only. It does not cover:

- Rustdoc (`///`, `//!`) that documents public API, including `# Examples` and `# Errors` sections.
- The `// Given` / `// When` / `// Then` comments in `tests/acceptance.rs`, which the Keeler workflow requires to mirror spec scenarios.

A `PreToolUse` prompt hook in `.claude/settings.json` checks every `Write`/`Edit` against this rule and rejects changes that add redundant comments.
