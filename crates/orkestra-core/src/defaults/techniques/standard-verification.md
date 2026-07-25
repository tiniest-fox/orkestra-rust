---
title: Standard Verification
description: >
  Test writing guidance — regression tests for bug fixes, branch coverage
  for conditional logic, integration tests for cross-module flows.
check: standard-checks
---

Bug fixes in pure functions always need a regression test. Write at least one test
that directly exercises the fixed code path.

New conditional branches always need tests. When you add if/else or match arms to a
pure function, write one unit test per branch. A function with 4 code paths needs 4 tests.

Unit tests live alongside the code they test in `#[cfg(test)]` modules. Integration
tests for cross-module flows belong in the `tests/` directory.

When fixing a bug, grep for existing tests asserting the old (broken) behavior. Update
them to assert the new correct value before submitting.

Don't mock internal modules — if you need to, the boundary is wrong. Only mock external
services (databases, network, filesystem) at system boundaries.

Test behavior, not implementation. Name tests after what they verify, not what code
they call.
