# #478 — test(certify): #460 lever 1 — the cover scales when its capacity is allowed to; two traps fixed first

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T13:03:32Z
- **Updated:** 2026-08-07T13:41:49Z
- **Closed:** 2026-08-07T13:33:38Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/478
- **Merged:** 2026-08-07T13:33:38Z
- **Merge commit:** eb3da7823473dd75f211c1ee72afa50328f009e0
- **Base / Head:** main / UOR-Foundation:measure/460-cover-scaling
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #460 (lever 2 landed in #477; this is lever 1, the last one).

## Why this lever stayed dark: the harness said PASS on one data point

Running `cover_scaling.rs` as it stood:

```
frac 13%: empty partition, skipped
frac 25%: empty partition, skipped
frac 50%: empty partition, skipped
PASS absolute monotone_regions=true ... predictive_ok=true (top1 0.1270 vs baseline 0.1270)
```

Condition 1 is "region count non-decreasing **in corpus size**" — evaluated over a single size, because `windows(2).all(..)` is vacuously true over one row. Condition 3 compared the baseline arm with itself. The run does not fail; it just says nothing, which is the #354 vacuous-green class in the harness whose only job is to decide this lever.

**Cause.** Nested prefixes need every prefix to contain both partitions. The `R4_STORIES` path partitions by `blake3(article id) % 5` and interleaves; the fallback was a contiguous tail split, so all held-out stories sat above the 80% mark and no smaller prefix had any. Fixed: interleaved 80/20 fallback, and an arm with fewer than two sizes now reports **INCONCLUSIVE** and can never print PASS.

**Second trap.** `DEFAULT_CAPACITY_REF_N = 400_000`; the committed fixture yields **400,006** train observations. At the anchor every scaled knob equals its base value, so at the largest size `scaled-k0` reduces *exactly* to `absolute` — identical regions (48), contrast (0.6217) and top-1 (0.1307) — and passes the exit rule by comparing itself with itself. Added `@ref50k` arms that lower the anchor to 50,000 so the law is exercised where the verdict is read.

## The shipped criterion does not scale — visible inside 500k

| train obs | 51,696 | 99,455 | 199,562 | 400,006 |
|---|---:|---:|---:|---:|
| `absolute` regions | 50 | 48 | 46 | 48 |

Flat and non-monotone over an 8× data range — **FAIL on condition 1**. Previously this was inferred from 500k (38 regions) vs 2.11M (14); it is now visible within one corpus.

## Capacity, not the criterion, is the binding knob

| arm | @51k | @99k | @199k | @400k | held-out top-1 @400k |
|---|---:|---:|---:|---:|---:|
| `absolute` | 50 | 48 | 46 | 48 | 0.1307 |
| `scaled-k0@ref50k` | 50 | 63 | 71 | **104** | **0.1713** (+4.1pp) |
| `relative+scaled@ref50k` | 46 | 65 | 81 | **110** | **0.1809** (+5.0pp) |

Both **PASS all three pre-declared conditions**: monotone regions; contrast 0.6649 / 0.6643 against a 0.2506 floor (above `absolute`'s 0.6217); and prediction not bought at a loss. Mean support per region falls 20,722 → 8,886 while top-1 rises.

The failures are informative. `mdl` scales (10 → 26 regions) but costs 1.7pp — capacity bought at a loss, exactly what condition 3 exists to catch. `relative` alone stays flat: making the *bar* track the data does nothing while `k0` and the region budget stay fixed.

## What it is worth — stated plainly

**Serving impact is capped near 0.15pp.** `exct.supported_record_fraction` is 0.9882 and the STAGES=5 run resolved 283 of 10,000 positions on the graph path, so +5pp on a path answering ~1–3% of positions is small. I pre-declared that cap on the issue before running.

The value is that it locates the bottleneck: **the cover is not what limits the engine — exact-context dominance is.** And the fix is cheap, already implemented, default-off, and now measured, so it is ready when a broad corpus drops exact-context coverage (#320, #433) instead of being a fresh unknown then.

It also revises a standing reading. #435 and the #460 STAGES=5 outcome both recorded the cover as simply not participating — "the absolute entropy floor rejects every split at this scale." True of the shipped *configuration*; not a property of the geometry.

## Pattern refinement

The README pattern says resolution has never helped. This is a boundary on it, not a counterexample: added resolution helps when a structure is *barely partitioned at all* — 48 regions for 400,006 records puts each region's emission near the global prior. The failed resolution levers all subdivided structures already resolved enough to be predictive. Resolution pays up to the point where a structure predicts, and not past it.

## Scope limits

`@ref50k` is a measurement device, not a proposed default — choosing a production anchor is a calibration question this run does not settle. Adoption of any non-`Absolute` criterion or scaled capacity is κ-affecting and needs the #407 ceremony. Measured to 400,006 train observations; whether growth and contrast hold at 2.11M is untested and is the run that would justify adoption.

## Gates

`cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `check_claim_wording.py`, `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`, and the `uor-r4-graph-certify` suite all green. The harness is `--ignored`, ~4 minutes for five arms, no teacher or checkpoint.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

