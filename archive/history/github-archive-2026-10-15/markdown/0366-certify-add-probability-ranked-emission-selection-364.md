# #366 — certify: add probability-ranked emission selection (#364)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T03:04:44Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-03T04:05:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/366
- **Merged:** 2026-08-03T04:05:22Z
- **Merge commit:** 42f6926fd47eab390e7f1c328d60dab7ef25e60e
- **Base / Head:** main / UOR-Foundation:graph-emission-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements #364 as an **opt-in ablation**, not a default change. Builds on #365
(the measurement).

## Change

`--emission-selection ratio|probability`, defaulting to `ratio` — the shipped
log-ratio ranking, reproduced byte-exactly. Same opt-in pattern as
`ScoringVariant` (#80) and `Smoothing` (#67). Changing selection changes artifact
bytes and κ, so it stays gated until the numbers justify flipping the default.

Only *which* tokens enter the list moves. Each entry still stores its log-ratio,
so `root_score + log-ratio = log P_region` still holds — the change makes that
identity apply to tokens that actually occur rather than to 64 rare ones.

## Ablation (fixture corpus, ratio → probability)

| metric | ratio (shipped) | probability | Δ |
| --- | ---: | ---: | ---: |
| emission mass coverage | 1.57% | **69.17%** | overlap 0.78% → 100% |
| graph: teacher in region emissions | 3.82% | **70.83%** | **18.5×** |
| graph: candidate recall | 62.97% | **75.98%** | +13.0 pp |
| **graph: bits/token** | 17.10 | **13.32** | **−3.78** |
| graph: top-1 | 1.96% | 2.90% | +0.94 pp |
| exact_context | 42.47% / 6.83 | 42.47% / 6.83 | unchanged |
| novel | 23.53% / 9.29 | 23.53% / 9.31 | +0.03 bits |

The pre-registered check holds. Mass coverage is the quantity #364 claims is
broken, and it moves 1.57% → 69.17% — so the accuracy movement is confirmation
of the mechanism, not a coincidence that happens to correlate.

`teacher_from_active` going 3.82% → 70.83% is the headline: the graph now
actually carries the answer.

## What this does NOT fix

Recorded plainly. Graph top-1 reaches 2.90%, still **below the 8.76% achieved by
disabling the residual entirely**. So even with the right tokens selected, the
term still costs top-1 — a residual **magnitude/calibration** problem separate
from selection. Bits improve more than disabling did (−3.78 vs −3.09), so
coherence is genuinely better, but 13.32 is still 6.5 bits above exact_context's
6.83.

Fixing selection was necessary and is not sufficient. The remaining gap is the
next question.

## Verification

`cargo fmt`, `cargo clippy -p uor-r4-graph-certify -p uor-r4-graph-cli
--all-targets -D warnings`, certify suite (11 files) and graph-cli suite (16
tests) green. Default path is unchanged, so byte-exactness of the shipped
artifact is preserved.

Related: #364, #365, #362.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5161876833 — 2026-08-03T03:07:03Z — auser

## Correction: the residual is still net-harmful. I made the wrong comparison.

The PR body states the fix improves coherence more than disabling the residual
does (−3.78 vs −3.09). That compared **across** selection rules. The correct
comparison is **within** each rule, against its own root-only baseline:

| selection | bits α=1 | bits α=0 (root only) | residual effect |
| --- | ---: | ---: | ---: |
| ratio (shipped) | 17.10 | 14.01 | **+3.09 worse** |
| probability | 13.32 | 12.00 | **+1.32 worse** |

Under **both** rules the residual costs bits relative to not applying it. The fix
reduces the harm from +3.09 to +1.32; it does not make the term helpful. Top-1
tells the same story — the α sweep still peaks at α = 0 (8.76%) under
probability selection, with α = 1 at 2.90%.

The root-only baseline itself moved (14.01 → 12.00) because the candidate set
changed, which is why the cross-rule comparison looked favorable and was
misleading.

### What the fix actually achieved

Real, and in the candidate set rather than the residual:

| metric | ratio | probability |
| --- | ---: | ---: |
| emission mass coverage | 1.57% | **69.17%** |
| teacher in region emissions | 3.82% | **70.83%** |
| graph candidate recall | 62.97% | **75.98%** |
| graph bits floor (root-only) | 14.01 | **12.00** |

The graph now supplies the answer instead of the bare prior doing it, and the
achievable floor improves by 2 bits. That is worth having on its own terms.

### What remains

Applying the residual still costs both metrics even with the right tokens
selected. Selection was one defect; residual **magnitude/calibration** is a
second, independent one. Median residual spread is 6.57 nats against a root
spread of 6.87 — comparable magnitudes, where a correction term should presumably
be smaller than the prior it corrects.

So #364 is necessary and not sufficient, and the honest statement of this PR's
value is "better candidate set, smaller residual harm," not "residual fixed."

I would still land it — the candidate-set gains stand independently — but the
default should not flip on this evidence, and the body's coherence claim is
withdrawn.



---
_Comment/review ids in this file: 5161876833_

