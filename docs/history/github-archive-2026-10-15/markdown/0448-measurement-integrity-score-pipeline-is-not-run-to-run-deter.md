# #448 — MEASUREMENT INTEGRITY: score pipeline is not run-to-run deterministic at cover induction

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-06T03:14:35Z
- **Updated:** 2026-08-06T05:01:48Z
- **Closed:** 2026-08-06T05:01:48Z
- **Labels:** bug
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/448
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Discovered while running #446 M1. **The same command, on the same corpus, produced different Gate C tables in two runs.**

Run A (02:28): cover `regions=10, mean_n=5536570` → Rule 1+2 **29.1%** / 7.9892 bits; Σ-cloud 0.6% / 19.7256; fwd-anchor 33.4%.
Run B (later): cover `regions=8, mean_n=4856412` → Rule 1+2 **26.4%** / 8.2674 bits; Σ-cloud 6.1% / 12.2525; fwd-anchor 31.1%.

Command in both cases: `r4 transformerless score --corpus-meta /tmp/c_meta.bin --corpus-recs /tmp/c_recs.bin --out ...`

**Why this is serious.** This project's entire discipline is pre-declared exit rules compared against recorded reference numbers. A 2.7pp run-to-run swing in the headline scorer is comparable to several of the effects we have accepted or rejected this week. Any comparison made ACROSS runs is now suspect and must be re-qualified; only within-run, same-table comparisons are safe. (Within-run orderings did stay stable and matched harness direction, so single-table verdicts survive.)

**Known-safe vs at-risk:**
- SAFE: every verdict computed from arms inside ONE Gate C table (M2/B′/draft/strict/two-sided all compare within-run).
- AT RISK: any claim of the form 'X at 500k vs Y at 2.11M' — including the falsifier-2 scale reading (36.5% → 26.1%). The scale gap is much larger than the observed variance, so it likely survives, but it is no longer a clean measurement.

**Next steps:** (1) find the nondeterminism source in `induce_cover` — thread scheduling in `build_observations_with_threads`, k-means seeding/tie-breaks, or HashMap iteration order reaching a reduction; the induction module docs claim k-means reductions stay ordered so thread count never changes emitted bytes, which this contradicts; (2) add a determinism regression test (same corpus twice → identical region count and cover κ); (3) once fixed, re-qualify the cross-run reference numbers we have been quoting.

## Comments (1)

### comment 5200301840 — 2026-08-06T04:13:06Z — Casey-allard

**REFUTED — my filing was wrong, and the cause was my own tooling.** Controlled test (PR #449): same binary, same corpus, frozen artifacts, scrubbed env, two back-to-back runs → `diff -r` empty; identical Gate C table and identical score.r4g1 κ. The pipeline is deterministic.

The 10-vs-8 region difference came from `--artifacts` defaulting to the shared mutable `/tmp/tless_artifacts.bin`. A background script I wrote (`/tmp/i435_scale_score.sh`) staged the 10k teacher container into that path and restored it afterwards; an overlapping score run read the wrong container. Isolated exactly — 500k container → 8 regions / 26.4%; 10k container → 10 regions / **29.1% / 7.9892**, reproducing the other row to the digit.

**Consequence for the record: the cross-run numbers I flagged as at-risk are NOT at risk.** The falsifier-2 scale reading (36.5% → 26.1%) and the reference numbers stand. Retracting that caveat.


---
_Comment/review ids in this file: 5200301840_

