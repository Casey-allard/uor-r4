# #353 — allocation census charges libtest runner allocations to measured sections (594 phantom bytes past 60s)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-02T23:02:53Z
- **Updated:** 2026-08-02T23:46:26Z
- **Closed:** 2026-08-02T23:46:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/353
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

`cargo test -p uor-r4-core --test allocation_census` fails on `main`: the R4G1
prediction kernels allocate in steady state, where the certified invariant is
zero.

```
[r4g1 kernels] predict_token + predict_distribution + predict_candidates + step × 10
    → 2 allocations, 594 bytes

assertion `left == right` failed: R4G1 runtime kernels must be allocation-free in steady state
  left: Census { allocations: 2, bytes: 594 }
 right: Census { allocations: 0, bytes: 0 }
```

Zero-alloc hot path is one of the kernel invariants the teacher-parity suite
certifies alongside the zero-multiply op census, so this is a published
guarantee currently false.

## Reproducing

```
cargo test -p uor-r4-core --test allocation_census
```

**Requires `.uor-models/compiled/smollm2-135m-instruct/` to be present.** Without
it the test passes vacuously in ~1.9s instead of running for ~100s — see the
companion issue on fixture-absent vacuous green. Check the runtime: a ~2s "pass"
means the assertion never executed.

## Characterization

Two observations narrow this down, both from the test's own structure
(`crates/uor-r4-core/tests/allocation_census.rs`):

1. **Not one-time lazy initialization.** The test runs a 10-iteration warm-up
   loop *before* the measured loop, which would absorb any first-call
   allocation.
2. **State-dependent, not per-call.** Inputs are fixed every iteration, but
   `step(&mut state, …)` mutates `state`, so successive iterations are not
   identical. 2 allocations across 40 calls (10 iterations × 4 entry points) is
   neither per-call (~40 expected) nor one-time (0 expected after warm-up).
   Something on an evolving-state path allocates occasionally.

That points diagnosis at `step()` and `RuntimeState`'s fixed-capacity buffers
rather than at the per-prediction scoring path.

Partial exclusion: the VP-tree's `to_vec()` / `collect()` calls in
`vp_tree.rs` sit in `from_graph` / `from_points` / `build` — construction time,
already accounted separately as the report-only 1548 allocations at
`R4G1Runtime::parse`. `query` writes into a fixed `&mut [u32; 8]`. `search_node`
has not been read, so the VP-tree is not fully ruled out.

## Age

Predates the fixture re-pin. Bisect was attempted over `fcd8595..34e1062`
(13 commits), but the intended good anchor `fcd8595` is already BAD, so the
regression is older.

Bisecting further back is unreliable rather than merely slow: earlier commits
cross the TLA5/TLA6/TLA7 artifact-era boundary, where the currently on-disk
`.uor-models` no longer matches what those commits expect, so a "BAD" result
there cannot be distinguished from a format mismatch. Direct diagnosis is the
better route, which is why the characterization above is more useful than a
culprit commit would have been.

It is plausible this has never passed with real artifacts present and has been
vacuously green in CI since the assertion was broadened in #164 (`5127c71`,
"broaden zero-allocation kernel coverage") — worth confirming as part of the
fix.

## Suggested acceptance

- The census reports 0 allocations / 0 bytes for the R4G1 kernel loop with
  artifacts present.
- A regression guard that cannot pass vacuously (see companion issue).


## Comments (2)

### comment 5160800754 — 2026-08-02T23:11:37Z — auser

## Correction to the characterization above, with attribution

Diagnostic runs localize this precisely, and contradict part of what I filed.

**Attributed:** it is `step`, on the **first call of the measured loop** — the
11th `step` overall, immediately after the 10-iteration warm-up. Every other
entry point and every other iteration reports 0.

```
[DIAG] iter 0 step → 2 allocs, 594 bytes
[DIAG] total across loop → 2 allocs, 594 bytes
```

**Correction:** I described this as "state-dependent, not per-call … something on
an evolving-state path allocates occasionally." That framing is wrong. It is a
single growth event at a threshold, not an occasional path. The mechanism is
visible in `engine.rs`:

```rust
state.record_token(token);
let context = state.token().as_slice();
let (pred_token, score) = self.predict_distribution(context, None, &mut node_scores);
```

`record_token` extends the token context every call, so `predict_distribution`
receives a context that grows 1, 2, 3 … Warm-up covers lengths 1–10; the first
measured call is the first at length 11. Something sized by context length grows
there. `TOKEN_STATE_CAPACITY` is 32, so the token ring itself is not the thing
growing — it is downstream of the context slice.

**Methodological note worth recording.** My first diagnostic restructured the
loop into 10× of each entry point instead of interleaving them. That made the
failure *disappear* — all 40 calls reported 0 allocations and the test passed.
The extra calls before the measured loop absorbed the growth. Anyone
investigating should keep the original interleaving and warm-up count and change
only measurement granularity; regrouping the calls silently hides the defect.

This also means the "2 allocations across 40 calls" arithmetic in the original
report should not be read as an allocation rate — it is one event, counted twice
because buffer growth surfaces as a fresh allocation of the new size (per the
census harness's own note on `realloc`).

Exact call site to follow from a backtrace-capturing allocator run.


### comment 5160827725 — 2026-08-02T23:19:17Z — auser

## Correction: this is a harness defect, not a runtime regression. The R4G1 kernels do not allocate.

I filed this as a violated zero-alloc invariant. That was wrong, and the title/body
above are wrong. A backtrace-capturing allocator names the actual source:

```
 12: <alloc::raw_vec::RawVec<test::types::TestDesc>>::grow_one
 14: <alloc::vec::Vec<test::types::TestDesc>>::push
 15: test::run_tests::get_timed_out_tests
 16: test::run_tests::<test::console::run_tests_console::{closure#2}>
 17: test::console::run_tests_console
 18: test::test_main_inner::<test::test_main_static::{closure#0}>
 19: test::test_main_static
 20: allocation_census::main
```

Both allocations belong to **libtest's own timeout bookkeeping** —
`get_timed_out_tests` growing its `Vec<TestDesc>` — at 512 and 82 bytes,
summing to exactly the reported 594.

### Mechanism

`CountingAlloc`'s gate is a process-wide `AtomicBool`. The test body runs on a
worker thread while libtest's console/runner loop runs on main. When a test
exceeds 60 seconds, the runner allocates to record it — and whatever measured
section happens to be open at that moment is charged for it.

The module docstring anticipates the general hazard:

> the allocator's gate and counters are process-wide, and libtest runs tests in
> parallel threads by default, so several measured tests could let one test's
> bookkeeping allocations land in another test's census

The mitigation chosen was "one single `#[test]` by design." That closes the
cross-*test* leak but not the cross-*thread* leak from the runner itself.

### Why the evidence looked like a state-dependent runtime bug

- **Attribution wanders.** One run blamed `step`, the next `predict_candidates`
  — identical build, identical inputs. It lands on whichever call is executing
  when the 60s timer fires.
- **It "vanished" under a restructured diagnostic.** Regrouping the calls made
  the run take 170s, so the watchdog fired outside a measured window and the
  test passed. That was timing, not workload.
- **The 60s message prints immediately before the failure** in the log — the
  clearest tell, and the one I should have read first.

### Interaction with #354

These are linked. The census only trips the watchdog when it is slow enough to
run for 60s, and it is only that slow when the artifacts are present. So:

- artifacts absent → ~1.9s → vacuous pass (#354)
- artifacts present → ~100s → crosses 60s → 594 phantom bytes → spurious fail

There was never a window in which this test was both running and correct.

### Actual fix

Make the gate thread-local so the census counts only the measuring thread's
allocations, leaving the runner's bookkeeping uncounted. Retitling accordingly.

Apologies for the noise — the original report asserted a false runtime defect.



---
_Comment/review ids in this file: 5160800754, 5160827725_

