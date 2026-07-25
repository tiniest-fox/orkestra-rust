---
title: Standard Review
description: >
  Code review framework — spawn specialized reviewer subagents based on
  change scope, synthesize findings, proportional rejection.
---

Review changed files across relevant dimensions: correctness, boundary violations,
naming precision, test coverage, and simplicity.

Spawn specialized reviewer agents for each dimension in parallel. Each reviewer
focuses on one concern and is blind to what others surface.

Adversarially verify findings before reporting. A finding that cannot be confirmed
with a concrete failure scenario should be dropped.

Synthesize results by ranking findings most-severe first. Deduplicate across
reviewers — the same issue found by two reviewers is one finding.

Rejection must be proportional. A MEDIUM finding in an otherwise clean change
warrants a note, not a full rejection. Reserve rejection for HIGH-severity issues
or patterns that would propagate if merged.

Report findings with: file path, line number, one-sentence summary, and a concrete
failure scenario (inputs → wrong output/crash).
