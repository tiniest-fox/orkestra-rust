---
title: Implementation Conventions
description: >
  Baseline implementation discipline — follow existing patterns, update docstrings
  when changing behavior, grep for mirrored string constants when renaming.
check: checks
disallowed_tools:
  - pattern: 'Bash(cargo test*)'
    message: Automated gate runs tests — use cargo check for quick compilation verification
  - pattern: 'Bash(cargo clippy*)'
    message: Automated gate runs clippy
  - pattern: 'Bash(cargo fmt*)'
    message: Automated gate runs formatting
---

Before writing new code, search for similar implementations in the codebase to understand
existing patterns, naming conventions, and error handling approaches. Follow them.

When renaming a constant, heading, or output field, grep for the old string before finishing —
it may be mirrored in schemas, doc comments, mock implementations, test assertions, and
hardcoded fallback values.

When changing how a function works, update every docstring that describes that behavior:
trait method docs, command docs, file-level headers, and inline comments.

Verify all production callers are wired when adding opt-in features. A feature that works
in tests but is never reached in production is dead code.

Don't add error handling, fallbacks, or validation for scenarios that can't happen.
Only validate at system boundaries (user input, external APIs).
