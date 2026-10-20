# #352 — bench: measure #330 isolated dot scan (3.82×)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T22:47:31Z
- **Updated:** 2026-10-09T13:09:09Z
- **Closed:** 2026-08-02T23:05:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/352
- **Merged:** 2026-08-02T23:05:55Z
- **Merge commit:** da09459f808faf3ae7da82f03852cde69eb1296d
- **Base / Head:** main / UOR-Foundation:issue-330-isolated-benchmark
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #351, which landed the compact one-term dot layout but left #330's
pre-registered **≥3× isolated microbenchmark** criterion unmeasured — the #330
thread had judged it "out of reach on i64×2 lanes without a storage-design
change," and #351 *was* that storage-design change, so it was the first real
test of that claim.

This adds the measurement surface, takes the number, and corrects the framing of
the figures #351 recorded.

## Result

**≥3× isolated: met at 3.82×.**

arm64 (M4 Max), release, min-floor over 8 runs at 20,000 iterations on an idle
machine (spread ≤2%), TLA6 fixture `smollm2-135m-instruct-tla6`. All three scans
cover identical work — one full K×D scan per stage, 1024 class rows — in a single
process, so they share fixture, build, and machine state.

| isolated dot scan | ns/scan | vs scalar |
| --- | ---: | ---: |
| scalar reference (`dot_score_plain`) | 211,712 | 1.00× |
| legacy two-term SIMD layout | 142,213 | 1.49× |
| compact one-term SIMD layout | 55,397 | **3.82×** |

The checked-in fixture reproduces independently at **3.79×** (min-floor over 6
runs), with the same 2.57× layout delta.

## Corrections to what #351 recorded

**The ≥2× end-to-end claim compared the wrong pair.** #351 reported 159,189 →
59,556 ns/token as clearing that criterion, but both sides are SIMD — that is a
layout delta, not a ratio against the scalar baseline the criterion names.

There is no usable scalar *serving* baseline: the scalar fallback in
`Runtime::dot_argmax` routes every table entry through the op-census kernel
(~295k counter increments per token), so timing it would inflate the baseline and
flatter the ratio. The criterion is therefore restated as a bound, using #330's
own op-census figure (dot scan ≈95% of steady-state serving cost): scalar serving
≥ the 211,712 ns scalar scan, against 57,425 ns/token measured. At face value
3.88×; the **≥3.69×** floor assumes the remaining ~5% is free. Either way ≥2× is
comfortably met — but as a labelled bound, not an unqualified ratio.

**The 159,189 ns/token figure is superseded.** It implies ~17µs of per-token
bundling for work that measures ~2.0µs here, and was a single 5,000-iteration
sample on a contended machine. The layout delta is now measured in-process
against a forced legacy build (2.57×) instead of across two commits.

Two corroborations fell out: the legacy layout's 1.49× isolated ratio
independently lands near #334's reported ~1.5×, and compact end-to-end minus
compact isolated leaves the ~2.0µs of bundling noted above.

## Measurement surface

Isolated timing needs to reach `dot_argmax` directly, so `simd::bench` exposes
the built tables read-only behind a default-off `bench-internals` feature. It
also reports which layout load-time selection chose: the benchmark prints
`simd_dot_layout=compact`, witnessing that the compact path is what was measured
rather than assuming the selection predicate engaged. (#330's own thread records
a dispatch-default bug class where a census silently exercised scalar — this is
the cheap guard against that.)

`DotTables::from_packed_forced_two_term` is gated to
`#[cfg(any(test, feature = "bench-internals"))]`; the deployed path only ever
calls `from_packed`. `from_packed` itself is refactored into shared
`build_single_term`/`build_two_term` helpers with no behavior change.

## Test

New proptest `compact_and_legacy_layouts_agree` asserts both representations
decode the same packed ABI to the same argmax over random weights and work
vectors — the load-layout counterpart to the existing scalar/SIMD output
witnesses. It runs under plain `cargo test`, not only under the bench feature.
The benchmark also cross-checks the two layouts before timing either.

`cargo fmt`, `cargo clippy --all-targets -D warnings` (both feature states), and
`cargo test -p uor-r4-core --lib` are green locally.

## Unrelated pre-existing failure found while verifying

`cargo test -p uor-r4-core --test allocation_census` **fails on main**: the R4G1
kernels (`predict_token`/`predict_distribution`/`predict_candidates`/`step`) do 2
allocations / 594 bytes where the invariant demands zero. Bisected to before the
dot-layout work — it reproduces at 34e1062 — so it is not from #330 and is not
addressed here.

It appears to be green in CI because the test resolves artifacts from
`.uor-models/compiled/` and passes vacuously in ~1.9s when they are absent
(versus ~100s when it actually runs) — the skip-to-vacuous-green case AGENTS.md
lists under "Things that bite". Not yet filed as its own issue; flagging it here
so it is on the record.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

