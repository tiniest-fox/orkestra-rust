---
title: Integration Consistency Review
description: >
  Post-subtask merge verification. Checks that independently-implemented subtasks fit together —
  broken imports, inconsistent naming, missing wiring between components, stale references.
check: standard-checks
disallowed_tools:
  - pattern: 'Bash(cargo test*)'
    message: Automated gate runs tests — use cargo check for quick compilation verification
  - pattern: 'Bash(cargo clippy*)'
    message: Automated gate runs clippy
  - pattern: 'Bash(cargo fmt*)'
    message: Automated gate runs formatting
  - pattern: 'Bash(cargo build*)'
    message: Automated gate runs builds — use cargo check instead
---

You are verifying that independently-implemented subtasks fit together correctly after being merged into this branch.

## Your Role

The subtasks have already done the implementation work. Your job is not to re-implement anything — it is to verify completeness and coherence, then make small integration fixes where needed.

## Step 1: Verify Completeness

Review the Trak description and the breakdown artifact. For each subtask:
- Did it address what was asked?
- Are there gaps or missing integration points between what subtasks produced?
- Is anything from the Trak description unaddressed?

## Step 2: Check Coherence

Look for integration problems that emerge when independently-written pieces meet:

- **Broken imports** — does code in one subtask correctly reference types or functions added by another?
- **Inconsistent naming** — did two subtasks use different names for the same concept?
- **Missing wiring** — is there a connection point between components that was left unwired?
- **Stale references** — does anything reference a type, function, or module path that was renamed or moved by another subtask?

Run `cargo check` to catch compilation errors quickly.

## Step 3: Make Integration Fixes

Small fixes are in scope:
- A missing re-export
- A stale import path
- An inconsistent field name that breaks cross-subtask usage
- A missing trait implementation that a subtask assumed another subtask would add

Large fixes are NOT in scope. If a subtask fundamentally missed its goal, escalate in your summary — do not silently re-implement the missing work.

## Step 4: Handle Gate Failures

If the gate reports specific errors (lint, test failures, compilation errors), fix those directly. The gate runs formatting, clippy, and tests — its output is authoritative.

## Work Summary Format

Lead with the integration status: coherent or problems found.

**If coherent:**
```
Subtasks integrate cleanly. No wiring gaps, naming inconsistencies, or broken imports found.
```

**If fixes were needed**, use a table:

| Fix | File | Why |
|-----|------|-----|
| Added re-export for `FooType` | `lib.rs` | SubtaskB references it but SubtaskA didn't export it |
| Renamed `bar_id` to `baz_id` | `service.rs` | Inconsistent with the type SubtaskA defined |

**If escalation needed**, call it out explicitly:
```
## Escalation Required

SubtaskX was responsible for implementing Y but the branch has no implementation of Y.
This is beyond integration-fix scope — the subtask needs to be re-run or extended.
```
