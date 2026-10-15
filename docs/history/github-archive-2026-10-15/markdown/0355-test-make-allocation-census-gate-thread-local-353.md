# #355 — test: make allocation census gate thread-local (#353)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T23:30:37Z
- **Updated:** 2026-10-09T13:09:09Z
- **Closed:** 2026-08-02T23:46:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/355
- **Merged:** 2026-08-02T23:46:25Z
- **Merge commit:** 9dae300bdb6d986e75115441d725f46d4d5ff454
- **Base / Head:** main / UOR-Foundation:issue-353-r4g1-alloc
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes #353.

## What was wrong

The allocation census gate was a process-wide `AtomicBool`, so allocations made
by **any** thread while a measured section was open were counted into that
section. libtest's runner thread allocates during the test: once a test passes
60 seconds, `test::run_tests::get_timed_out_tests` grows a `Vec<TestDesc>`.

```
 12: <alloc::raw_vec::RawVec<test::types::TestDesc>>::grow_one
 14: <alloc::vec::Vec<test::types::TestDesc>>::push
 15: test::run_tests::get_timed_out_tests
 17: test::console::run_tests_console
 19: test::test_main_static
 20: allocation_census::main
```

512 + 82 = exactly the 594 bytes that were failing the "R4G1 runtime kernels
must be allocation-free in steady state" assertion.

**The R4G1 kernels never allocated.** I originally filed #353 as a violated
zero-alloc invariant; that was wrong and is corrected in the issue.

## Why it read as a runtime bug

- Attribution **wandered** between `step` and `predict_candidates` across
  identical builds and inputs — it tracked whichever call was executing when the
  60s timer fired.
- A restructured diagnostic made it **disappear**: regrouping the calls pushed
  the run to 170s, so the watchdog fired outside the measured window and the test
  passed.
- The `running for over 60 seconds` line prints immediately before the failure in
  the log.

The module docstring already anticipated the cross-*test* form of this hazard and
mitigated it with a single `#[test]`. That closes the cross-test leak but not the
cross-*thread* leak from the runner itself.

## Fix

Gate and counters become thread-local, so only the thread inside `measure` is
counted. `try_with` keeps allocations during TLS teardown from panicking.

## Verification

Every other census figure is byte-identical before and after — artifacts
13/530884, store 121405/11389105, r4g1 parse 1548/258148, add_evidence
563/51088, op census and generated tokens unchanged. The only movement is the
R4G1 kernel line: **2 allocations / 594 bytes → 0 / 0**.

The passing run still logs `running for over 60 seconds`, so the watchdog is
still firing — the fix stops it being *counted*, it doesn't just make the test
faster.

## Regression guard

Added a guard that allocates 4 KB on a helper thread and asserts the census stays
zero. The helper is spawned **outside** the measured window on purpose:
`thread::spawn` allocates on the spawning thread (boxed closure, `JoinHandle`,
thread name), and those allocations legitimately belong to the measured thread —
my first attempt got this wrong and failed for that reason. Coordination is
atomic-only, which does not allocate.

Verified to fail against the pre-fix harness (`1 allocation, 4096 bytes`) and
pass after — in 0.00s, versus the ~100s run the real symptom required.

## Related

#354 (fixture-absent vacuous green) is the other half. The two interlock: with
artifacts absent the test finishes in ~1.9s and passes without asserting
anything; with them present it runs ~100s, crosses 60s, and failed spuriously.
There was no configuration in which this test was both running and correct.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

