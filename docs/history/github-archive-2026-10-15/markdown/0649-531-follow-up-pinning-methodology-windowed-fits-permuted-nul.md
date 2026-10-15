# #649 — #531 follow-up: β-pinning methodology — windowed fits + permuted-null exponent arm

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-12T22:06:10Z
- **Updated:** 2026-08-13T03:23:15Z
- **Closed:** 2026-08-13T03:23:15Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/649
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Research-available methodology note for when #531's compute-bound past-knee runs happen. Not a mechanism change.**

## Basis (measured, historical program)
The historical routing capacity law `eff_buckets = 2.957 x K^0.572` (norm-invariant) **steepened to α=0.657** over K=600–5000, with a permuted-null exponent of 0.816 (R²=0.991); the *ratio* stayed stable at 2.6–2.8x because the null grew faster. A single global exponent underestimated by up to +65% at K=5000. Interpretation: measured knee-approach behavior, visible only against a null arm.

## Proposal for #531 β pinning (provisional 0.5)
1. Pre-register **windowed exponent fits** — report restricted-window AND full-range β, don't collapse to one.
2. Add a **permuted/null exponent arm** so saturation reads as ratio-vs-null rather than absolute-law failure.
3. Pre-register a **Δβ drift threshold** (historical program used Δα ≤ 0.10).

Expect β to be scale-window-dependent; a single global β is the known failure mode. #531 is closed/compute-bound; this is the methodology to apply when the multi-hour past-knee observe runs are executed.


## Comments (1)

### comment 5275565274 — 2026-08-13T03:23:14Z — Casey-allard

Closing as **methodology recorded, applies to a future #531 run** (this is a pre-registration note for when #531's compute-bound past-knee runs happen, not a standalone change).

Recorded for the next #531 β-pinning pass: (1) windowed exponent fits — report restricted-window AND full-range β, don't collapse to one; (2) a permuted/null exponent arm so saturation reads as ratio-vs-null rather than absolute-law failure; (3) a pre-registered Δβ drift threshold (historical program used Δα ≤ 0.10). Basis: the historical routing-capacity law steepened α 0.572→0.657 over K=600–5000 while the null grew faster (ratio stable 2.6–2.8×); a single global exponent underestimated by up to +65% at K=5000. Reopen/apply when #531 executes past-knee.


---
_Comment/review ids in this file: 5275565274_

