# Compiled relative-query transport: folded action, inverse-frame selection — September 24, 2026 UTC

This record delivers an orphaned, pushed-but-unmerged workstream (`codex/compiled-relative-query-20260924`,
authored by the prior agent). It was CI-verified only because the desktop link was unavailable, and **no PR had ever
merged**. The run lead cherry-picked its ten commits onto `d71a3143`, verified it locally, and applied the three
review-recommended corrections. It is an **exposed research component**, not a model promotion. The geometric,
fully transformerless objective is unchanged.

Plan: [compiled-relative-query-plan-2026-09-24.md](compiled-relative-query-plan-2026-09-24.md).

## What it does

`CompiledRelativePath { mapping: [i8; 4], reject_min_mask: u8 }` reduces a learned Q8 relation path to a single
signed permutation plus the coordinates whose intermediate negation would overflow `i32::MIN`.
`RelativeActionModel::select` now compiles the path once per call, moves the query into the inverse frame, and
scans unchanged candidate key identities:

> For a signed permutation `R`, `‖q − R·k‖₁ = ‖R⁻¹q − k‖₁` (signed permutations are exactly the linear L1
> isometries; the identity **fails** for a general orthogonal `R`). Thus `select` is `O(L + N)` instead of `O(N·L)`,
> with selection, tie-breaking (first minimum wins), payload identities and Q8L1 model bytes unchanged.

A 274-line standalone test target compares the compiled path against the unchanged sequential `act` oracle; a
narrow, read-only GitHub-hosted job (`.github/workflows/relative-query-check.yml`, 5-minute cap, no secrets, no
model downloads) re-runs it. The compiled plan is a transient execution plan; it compiles no learned parameters and
adds no artifact format version.

## Verification

- **Mathematics (independent):** the L1-isometry identity holds exactly for signed permutations (and fails for
  general orthogonal maps); `reject_min_mask` is **exactly** the rejection set of the sequential checked path,
  including cancelling negations (two negations still reject `i32::MIN` at the first); widening the query to `i64`
  is necessary for the inverse-frame formulation and sufficient; the `u64` sum cannot overflow. Verdict: the algebra
  and rejection set are exact.
- **Computer science (independent):** `O(L+N)` confirmed (`compile` O(L) + candidate scan O(N)); **no correctness
  fix required**; tie-breaking and error precedence preserved; the success path allocates **0**; rebase onto
  `d71a3143` is clean (disjoint files); the test is cargo-runnable (`uor-r4-wasm-router`).
- **Local execution (run lead):** `cargo fmt --check` clean; **18/18** standalone tests pass on the pinned
  `rustc 1.97.1`; `EXHAUSTIVE_PATHS=4681` (all paths of length 0–4 against the **independent** sequential oracle);
  `COMPILED_PATH_BYTES=5`; `SUCCESS_PATH_ALLOCATIONS=0`; `m.to_bytes()` unchanged.

**Measured CPU cost** (ns/query, 11 rounds, debug build — a microbenchmark, not a serving claim):

| path length | candidates | sequential | integrated `m.select` | prepared (compile-once) |
| ---: | ---: | ---: | ---: | ---: |
| 32 | 256 | 37,389 | 1,451 | 988 |
| 32 | 16 | 2,328 | 460 | 67 |
| 32 | 1 | 148 | 404 | 9 |
| 6 | 256 | 7,821 | 1,135 | 1,014 |

The compiled path wins decisively for many candidates and is **slower for a single candidate** (compile + `act`
overhead) — reported honestly.

## Corrections applied by this delivery

The CS review recommended three (non-blocking) changes, all applied: (i) a new test
`long_paths_match_sequential_where_a_coordinate_is_first_negated_after_step_four` (exhaustive at lengths five and
six plus hand-picked late-negation paths), because the length-four test cannot reach a coordinate first negated
after step four; (ii) `cancelled_signs` now compares against the independent `sequential` oracle rather than the
now-delegating `m.select` (which had become circular); (iii) the plan's "compile once" wording now states that the
retained selector recompiles per call and the reusable plan is the "compile once" object.

## Limits

- Scope is bounded typed relative selection over supplied query/key vectors with exact ownership — **not** learned
  natural-language attention, general language, reasoning, or geometric superiority. The Q8 learner's established
  tie with an ordinary signed-permutation control is unchanged.
- Microbenchmark only (debug, single machine); no latency or energy claim. The compiled plan is not cached across
  selectors' calls.
- The prior orphaned branch `codex/compiled-relative-query-20260924` on origin is **preserved**; delivery is a new
  branch cherry-picked onto `d71a3143`.

## Resources

Local: `cargo check`/build and one 18-test run (build ≈ 7–8 min from a shared target); no model execution, no
training, no artifact change; two threads, one cargo process at a time. Charge **+600,000 ms**; cumulative ledger
**450,039,320 / 455,500,000 ms**. Evidence file: [`docs/evidence/compiled-relative-query-2026-09-24.json`](../evidence/compiled-relative-query-2026-09-24.json).

## One recommended next milestone

Resume the forward line: **make the span curriculum matched and fix the multi-token copy gap** — implement the
declared separate span RNG so `max_span ∈ {3,8}` arms are exactly data-matched; add in-range 3-token payloads and a
length-only control to separate "payload length ≥ 3 tokens" from "out-of-range value"; extend the copy/stop-length
curriculum; and bind the changed runner/module hashes into receipts. Then widen broader-source dialogue/code and
complete-session work.
