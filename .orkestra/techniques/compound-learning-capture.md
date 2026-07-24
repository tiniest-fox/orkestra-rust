---
title: Compound Learning Capture
description: >
  Passive-first scan of what a Trak revealed (confusion, rejection patterns, repeated check
  failures, prompt/skill gaps) and place any genuine learning at its most local, discoverable
  source. Most Traks need zero action from this Technique.
---

Your goal: make every future Trak faster by capturing what this Trak taught us. Not just what went wrong — what took time, what was confusing, and what could have been prevented. Most Traks are clean — don't manufacture insights.

## Critical Rules

1. **Never modify code.** Update documentation, comments, agent prompts, and skills only.
2. **Most Traks need no action.** Clean Traks are the norm.
3. **Place at the source.** Learnings go in the nearest CLAUDE.md to the code they describe. Only meta-guidance about *how* to work belongs in `.orkestra/agents/worker.md`.
4. **Never more than 3 CLI calls per Trak.** Most need zero.

## Triage Protocol

### Tier 1 — Passive Scan (always, no CLI calls)

Read the artifacts you've been given (plan, summary, check results, verdict) and activity logs. Look for:

- **Compound notes** (`compound-notes:*` resources) — direct signals from prior stages; treat as highest-priority input
- Meaningful notes from workers or reviewers (confusion, workarounds, failed approaches)
- Multiple activity log entries per stage (suggests rejections or retries)
- Check failures in check_results
- Reviewer observations about patterns, anti-patterns, or documentation gaps

**If everything is clean AND activity logs show single iterations per stage AND no compound-notes resources → stop here.**

### Tier 2 — Active Investigation (if signals found)

Run `bin/ork trak show {{task_id}} --iterations --pretty` to get the full iteration history. Look for: rejection cycles, repeated check failures, long iteration durations, integration failures.

### Tier 3 — Targeted Log Review (problem stages only)

Run `bin/ork logs {{task_id}} --stage <stage> --pretty` for specific problem stages. Look for: agent confusion (reading many files for patterns that should be documented), repeated failed approaches, missing conventions.

**Hard cap: never more than 3 CLI calls total across Tier 2 and Tier 3.**

## What to Look For

**Documentation errors** (highest priority) — incorrect docs actively mislead future agents: wrong CLAUDE.md guidance, misleading code comments, outdated patterns, wrong file/function references.

**Efficiency gaps** — what took time that documentation could prevent: agent exploring to find a pattern, worker discovering structure by trial and error, information existing but not where agents look first.

**Rejection patterns** — why was work rejected? Could better guidance prevent it?

**Check failures** — recurring lint/test failures indicating missing conventions.

## Where to Place Learnings

1. Crate-specific pattern → that crate's `CLAUDE.md`
2. Frontend pattern → `src/CLAUDE.md`
3. Tauri pattern → `src-tauri/CLAUDE.md`
4. Domain patterns with code examples → `.claude/skills/<name>.md` (use `bin/update-skill`)
5. Cross-cutting worker discipline → `.orkestra/agents/worker.md` (sparingly — only meta-guidance)
6. Reviewer criteria → `.orkestra/agents/reviewer-instructions.md`
7. Project-wide architectural decision → root `CLAUDE.md` (update existing sections)
8. Code-level "why" → code comment in the relevant file

**Anti-pattern:** Dumping everything into `worker.md`. If a finding is about a specific crate or directory, it belongs in that location's `CLAUDE.md`.

## Editing Rules

- **Consolidate, don't append.** Merge findings into existing sections if related guidance already exists.
- **Remove stale entries.** If an entry references code that no longer exists or a pattern that changed, delete it.
- **Skill edits require `bin/update-skill`.** Use `bin/update-skill <name> write`, `patch <start> <end>`, or `delete <start> <end>`.
- **No HTML comment markers.** Learnings should be indistinguishable from hand-written guidance.
- **Size discipline:** Individual entries: 1-3 paragraphs + optional code block. If you need more, it belongs in a skill or docs/ file.

## What NOT to Add

- Patterns already documented (grep first)
- Observations obvious from reading the code
- One-off fixes that won't recur
- Architecture descriptions (that's what crate CLAUDE.md files are for)
