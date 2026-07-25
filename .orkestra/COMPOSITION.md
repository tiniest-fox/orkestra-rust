# Composition Playbook

This document is the decision guide for the composer agent. It explains which techniques to select for a given Trak and how to combine them across steps. Technique names map directly to files in `.orkestra/techniques/`.

## Universal Techniques

Two techniques apply to nearly every implementation step:
- **`implementation-conventions`** — baseline implementation discipline (pattern-following, docstring hygiene, grep checks)
- **`standard-verification`** — test writing guidance and quality check separation

Include both on any step that produces code.

---

## When to Use `requirements-discovery` vs. the Bootstrap Chat

**Full `requirements-discovery` technique** — when any of these are true:
- The Trak description is ambiguous or underspecified
- The request spans multiple concerns or deliverables
- Success criteria haven't been stated
- Medium or larger scope Traks where mis-scoping is expensive

**Bootstrap chat (no technique, just a short clarifying exchange)** — when:
- The Trak is already clear and specific
- The user has described exactly what they want
- Scope is tightly bounded (single-file fix, config change)

Never skip requirements entirely for medium+ scope Traks.

---

## Bug Reports Without Clear Repro

Use a two-step investigation/fix flow:

**Step 1 — Investigation:**
- Technique: `red-green-investigation`
- Check: `expect-test-failure` (gate passes when the new test *fails*, proving the bug exists)
- Model: `opus`

**Step 2 — Fix:**
- Techniques: `regression-safety` + `implementation-conventions` + `standard-verification`
- Check: `standard-checks` (gate passes when tests pass)
- Model: `sonnet`

The investigation step's failing test is the spec for the fix step. The fix agent must not touch the investigator's test — only make it pass.

---

## Frontend Changes

Any step that adds or modifies UI components needs:
- **`storybook-story`** — covers each visual state (default, loading, error, empty)
- **`implementation-conventions`** — includes the frontend state scope rules (per-operation loading state, error display scope)
- **`standard-verification`** — regression tests for new conditional branches

For the review step, use `standard-review` as usual.

---

## The Review Convention

**Always use `standard-review` as the review step technique.** Never compose multiple review techniques.

`standard-review` self-sizes its reviewer panel based on change scope:
- Trivial changes: single focused pass
- Targeted changes: subset of specialist reviewers
- Cross-cutting changes: full panel

For subtask reviews (the `hotfix` flow), `standard-review` switches to single-pass mode — no subagent spawning. The `subtask-reviewer.md` agent already handles this; the technique still provides the scoping and verdict format.

---

## Prompt / Agent Changes

Use a fixed three-step flow. Each step has its own dedicated technique:

| Step | Technique | Model |
|------|-----------|-------|
| Discovery | `prompt-investigation` | `opus` |
| Writing | `prompt-writing` | `sonnet` |
| Review | `prompt-review` | `opus` |

No need to combine techniques within a step — each technique covers its step fully. The investigation step produces a requirements document; the writing step implements it; the review step verifies requirements coverage and prompt engineering quality.

---

## Docs-Only Traks

Use the documentation pipeline. Each step has its own technique:

| Step | Technique | Check | Model |
|------|-----------|-------|-------|
| Research | `docs-research` | — | `opus` |
| Writing | `docs-writing` | `docs-checks` | `sonnet` |
| Review | `docs-editorial-review` | `docs-checks` | `opus` |
| Compound | `compound-docs-learning-capture` | — | `sonnet` |

For the compound step, use `compound-docs-learning-capture` — not the code-focused `compound-learning-capture`. The docs variant has docs-specific placement rules and signal detection (disambiguation flags, component requests).

---

## The Decomposition Heuristic

**When to decompose:** Only create a breakdown step for Traks complex enough to need parallel or sequential subtasks. Simple Traks (single-file fix, config change, one-concern feature) skip decomposition entirely.

**Breakdown step technique:** `vertical-slice-decomposition`
- Model: `opus`
- The technique enforces vertical slices (end-to-end behavior) over horizontal layers
- It includes the integration rule: if Slice A creates a method and Slice B calls it, one slice must own the wiring — never leave it ambiguous
- Always includes a final integration/verification slice

**After subtask completion**, if the parent Trak is the integration worker, use `integration-consistency-review` to verify that independently-implemented subtasks fit together. This is not needed for single-step Traks.

---

## Compound / Learning Capture

Every workflow should end with a compound step to capture what the Trak taught.

- **Code workflows** → `compound-learning-capture` (passive scan first; most Traks need no action)
- **Documentation workflows** → `compound-docs-learning-capture` (same protocol, docs-specific placement)

Both use `model: sonnet`. Neither modifies code — only documentation, agent prompts, and skills.

Hard cap: never more than 3 CLI calls per compound step.

---

## Quick Reference

| Technique | Check | Model | Primary use |
|-----------|-------|-------|-------------|
| `implementation-conventions` | — | (default) | Any step that produces code |
| `standard-verification` | — | (default) | Any step that writes tests |
| `requirements-discovery` | — | (default) | Scoping ambiguous Traks |
| `vertical-slice-decomposition` | — | `opus` | Breakdown step for complex Traks |
| `red-green-investigation` | `expect-test-failure` | `opus` | Bug investigation step |
| `regression-safety` | `standard-checks` | (default) | Bug fix step |
| `storybook-story` | — | (default) | UI component visual coverage |
| `standard-review` | — | `opus` | Review step for all code changes |
| `prompt-investigation` | — | `opus` | Prompt iteration: discovery |
| `prompt-writing` | — | `sonnet` | Prompt iteration: writing |
| `prompt-review` | — | `opus` | Prompt iteration: review |
| `docs-research` | — | `opus` | Documentation: research |
| `docs-writing` | `docs-checks` | `sonnet` | Documentation: writing |
| `docs-editorial-review` | `docs-checks` | `opus` | Documentation: review |
| `compound-learning-capture` | — | `sonnet` | Post-code-Trak learning |
| `compound-docs-learning-capture` | — | `sonnet` | Post-docs-Trak learning |
| `integration-consistency-review` | `standard-checks` | (default) | Post-subtask integration verification |
