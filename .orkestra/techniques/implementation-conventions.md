---
title: Implementation Conventions
description: >
  Follow existing codebase patterns, module structure (interactions/types/interface/service/mock),
  naming, and error-handling conventions. Grep for mirrored string constants and stale docstrings
  before finishing. Present in nearly every code-touching step.
---

Follow the established patterns in this codebase rather than inventing new ones. Consistency matters more than theoretical perfection.

## Module Structure

Assemble from these building blocks — not every module needs all five:

| Building Block | File | When |
|---|---|---|
| Interactions | `interactions/{domain}/*.rs` | Always — one file per operation, one `execute()` entry point |
| Types | `types.rs` | When the module has its own error types or domain models |
| Interface (trait) | `interface.rs` | When you need polymorphism or dependency injection |
| Service | `service.rs` | Thin dispatcher grouping interactions behind a trait |
| Mock | `mock.rs` | When callers need a test double (behind `testutil` feature flag) |

**Reference:** `crates/orkestra-git/` (all five layers), `crates/orkestra-schema/` (minimal: types + logic only).

## Implementation Rules

- **One `execute()` per interaction.** Always the first function in the file. Private helpers go below in a `// -- Helpers --` subsection.
- **Service is a thin dispatcher.** Holds shared state, delegates each method to one interaction's `execute()`. No business logic.
- **No separate utilities layer.** If multiple interactions need shared logic, one calls the other, or extract an interaction.
- **Errors propagate with `?`.** Return `Result`. Only swallow errors when there's an obvious fallback the caller wouldn't care about.

## Before Starting

Search for similar implementations before writing new code:
- How are similar features structured?
- What naming conventions are used?
- What error handling patterns exist?

```bash
grep -r "similar_function_name" crates/
```

## Naming

- No `handle`, `process`, `do`, `manage`, `data`, `info`, `utils`
- Verbs for functions, nouns for data
- Longer descriptive name beats short ambiguous one
- Rename when behavior changes

## Before Finishing

**Grep for mirrored string constants.** When renaming a section heading, concept name, or output field in a prompt template or schema, search for the old string — it may be mirrored in JSON schema descriptions, trait doc comments, mock return values, and test assertions:

```bash
grep -r "Old Name" .
```

**Update docstrings when changing behavior.** When changing how a function works (sync → async, `--ff-only` → `--rebase`, `Option` → `Result`), update every `///`, `//!`, and inline comment that describes the old behavior. Grep the function name and skim every doc block mentioning it.

## File Structure Conventions

Three levels — use what the file needs:

| Level | Syntax | Use for |
|---|---|---|
| File header | `//!` | Every file — one-sentence purpose |
| Section | `// ====` (76 chars total) | Major divisions (types, impl blocks, tests) |
| Subsection | `// -- Name --` | Grouping methods by domain within an impl block |

Tests always get their own `// ====` section.

## Rust Conventions

- `pub(crate)` for items only consumed within the crate; `pub` only for genuine cross-crate API
- `debug_assert!` only for architecturally unreachable states; `assert!` for everything else
- Compiled regex: use `static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("...").unwrap())` — never construct inside a function called repeatedly
