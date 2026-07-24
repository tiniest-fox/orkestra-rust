---
title: Vertical Slice Decomposition
description: >
  Research the codebase and split a Trak into independently-implementable, end-to-end vertical
  slices (not horizontal layers), each with clear interfaces and dependencies. Err toward routing
  here whenever a request bundles more than one plausible deliverable — this Technique's own
  research is what confirms whether a real interface seam exists.
model: opus
---

Your job is to analyze the codebase and split a complex Trak into independently-deliverable vertical slices. Each slice should deliver working end-to-end behavior, not just a code layer.

## Why Vertical, Not Horizontal

**Bad (horizontal):** Types Trak → Service Trak → API Trak → UI Trak → Tests Trak
- Each slice "succeeds" independently while the feature is broken end-to-end
- Cross-cutting changes (new struct fields, new enum variants) break sibling slices
- No single slice owns "the feature works"

**Good (vertical):** "Merge flow works end-to-end" → "PR flow works end-to-end" → "UI integration"
- Each slice delivers testable behavior through the system
- If a slice adds a method, it also wires the method into whatever calls it
- Tests exercise the actual execution path (e.g., the orchestrator tick loop)

## Research Phase

Before proposing any decomposition, study the codebase:

1. **Find analogous implementations.** How does the codebase handle similar features? Read the actual code — understand lifecycle management, error handling, and testing patterns, not just file names.
2. **Identify reusable infrastructure.** Which traits, services, types, and utilities already exist? New code should compose existing building blocks. If you find yourself designing something the codebase already has, stop and use the existing version.
3. **Map integration points.** Identify the exact traits and interfaces the new code must implement or consume. Note specific function signatures.
4. **Confirm whether a seam exists.** The description may bundle multiple deliverables, but the actual code may not cleanly separate. Your research here is the source of truth — don't assume separability from the description alone.

## Decomposition Output

For each proposed slice:

- **Title**: Clear, specific action ("Add rate limiting middleware to API layer")
- **What it delivers**: The working end-to-end behavior, not just the layer
- **Dependencies**: Which slices must complete first
- **Integration point**: What this slice produces that others depend on, and what it consumes

**The integration rule:** If Slice A creates a method and Slice B calls it, one must own the wiring. Never leave "who wires this?" ambiguous.

## When Horizontal Slicing Is Acceptable

Foundation layers that genuinely must exist before anything else (database migrations, trait definitions with no callers yet) may be horizontal. But the first slice that implements behavior on top of the foundation must wire the full path end-to-end.

## Minimum: One Verification Slice

Always include an explicit integration/verification slice at the end that exercises the full path through the system and confirms the slices fit together.
