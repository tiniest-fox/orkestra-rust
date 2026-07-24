---
title: Standard Verification
description: >
  Baseline correctness floor for code changes: lint, build, and existing test suite must pass;
  write regression tests for bug fixes and new conditional branches. The near-universal
  verification technique for ordinary code changes.
---

Verify that your changes are correct and that future regressions are covered.

## The Baseline Floor

All of the following must hold before work is complete:

1. Code compiles without errors
2. Existing tests pass
3. No new lint warnings introduced
4. New behavior is covered by tests (see rules below)

The automated gate runs the full suite — your job is to write the right tests before it does.

## When to Write Tests

**Bug fixes always need a regression test.** A pure function (no side effects, deterministic) is trivial to test. Write at least one test that directly exercises the fixed code path. This is the most common cause of rejection on small fixes.

**New conditional branches always need tests.** When you add `if/else` or `match` arms to a pure function — routing logic, header selection, format dispatch — write one unit test per branch. A function with 4 code paths needs 4 tests. Untested branches are unverified behavior regardless of how obvious the logic looks.

**Handlebars template conditionals require tests for both branches.** When adding `{{#if field}}...{{else}}...{{/if}}` to a `.md` template, add tests that render with a value triggering the `if` branch AND a value triggering the `else` branch. Handlebars treats empty arrays and empty strings as falsy.

**Cargo feature flags that affect runtime behavior need regression tests.** Features like `preserve_order` or `arbitrary_precision` on `serde_json` change library behavior globally. Add at least one test that would fail if the feature were removed.

## Where Tests Live

- **Unit tests**: `#[cfg(test)]` module in the same file as the code
- **E2e tests**: `crates/orkestra-core/tests/e2e/` — use `TestEnv` with real SQLite and `MockAgentRunner`
- **Service docker tests**: `#[ignore]` tests inside `mod docker` in `tests/e2e.rs` for `crates/orkestra-service/` interactions that call `docker exec`

Any meaningful logic, core behavior, or cross-module flow must appear in an e2e test.

## Key Testing Principles

- **Drive the orchestrator**: Use `ctx.advance()` to test behavior, not direct API calls
- **Mock minimally**: Only mock external services (agents, LLM calls, GitHub API). Use real SQLite, git, worktrees
- **Test behavior, not implementation**: Name tests after what they verify, not what code they call

## Quick Sanity Check

`cargo check` catches type errors immediately without running the full suite. Use it as a fast feedback loop, not a substitute for the gate.

Note: `cargo check` skips `#[cfg(test)]` blocks — test-only type errors are invisible until the full run. If you write tests that construct services, annotate the binding with the trait type:

```rust
let store: Arc<dyn WorkflowStore> = Arc::new(InMemoryStore::new());
```
