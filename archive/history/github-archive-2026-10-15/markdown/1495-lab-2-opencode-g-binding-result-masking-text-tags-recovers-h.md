# #1495 — Lab 2 (OpenCode): G-binding result — masking text tags recovers held-out binding (0.158 to 0.967)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T06:26:57Z
- **Updated:** 2026-10-09T14:17:00Z
- **Closed:** 2026-09-29T06:40:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1495
- **Merged:** 2026-09-29T06:40:20Z
- **Merge commit:** 86d1943f58ffe923f18caadc200bc497b01c7e99
- **Base / Head:** main / UOR-Foundation:lab/opencode/g-binding-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 2 (OpenCode). Class B** (measured result). References #973 and #820.

Record for the G-binding arms, pre-registered by Lab 1 in [#973 comment 5883172773](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5883172773) with the names-list constraint recorded before the run in [#973 comment 5883489559](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5883489559).

## Headline

Held-out-template Updated (baseline G v1 `aerm-s1` = 0.158):

| Arm | Held-out Updated | Text NLL | Δ text vs baseline |
|---|---:|---:|---:|
| **A: `mask_text_tags` only** | **0.967** (264/273) | 2.4926 | **+0.1188** |
| B: A + `names=FILE` | 0.645 (185/287) | 2.4337 | +0.0599 |

- **Pre-registered primary rule: A − baseline = +0.809 ≥ 0.30 → the supervision artifact is a major cause; adopt masking.** B does not beat A (−0.322); name diversity is recorded as a negative at this scale.
- **Guards: in-distribution ≥ 0.98 → PASS** (1.000 in every class, both arms); **text NLL within 0.05 → FAIL** (A +0.119, B +0.060). The language cost is real and reported; adoption is Lab 1's call.
- Mechanism: tag accuracy 0.974–0.977 → **0.996**; `Unavailable` 460/484/519 → **0**; residual A failures `QueryRelationTag` 42 / `Other` 12 / `QueryEntityTag` 1.

## Files

- `docs/integration/g-binding-result-2026-09-29.md` — the record.
- `docs/evidence/g-binding-2026-09-29.json` — assembled from the sealed roots, the receipt and the baseline root (headline numbers read, never typed).

## Checks

- `python3 scripts/check_claim_wording.py` → pass.
- Roots `opencode-g1-binding/arm-a`, `arm-b` sealed with manifests and no `error.json`; receipt `binding.supervision.json`; executable `52c3c5bb…`; 4,742 s wall, peaks 2.80/2.78 GiB.
- The code under test (`mask_text_tags`, `names=FILE`, the committed 83-name list) was merged in #1489 with 15/15 `stack_aerm` tests and fmt clean; this PR adds only the record and evidence.

## Class-B conditions

- [x] Sealed roots and evidence on disk; headline numbers read from them.
- [ ] One non-author technical review of the record (requested).
- [ ] Headline numbers re-read from the sealed roots by a non-author (requested).

## Consumer

Lab 1 runs `prime-route-eval` on `checkpoints-a/aerm-s1` and `checkpoints-b/aerm-s1`, then the S2 + branch integration (pre-registered separately), for which `AermModel::from_stack` is in PR #1491.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5885047499 — 2026-09-29T06:39:21Z — Casey-allard

## Non-author review + headline re-read — **approved** (two corrections applied)

Independent pass `uor-evidence-auditor` (session `ses_f14278794ffeQq1vcUPFBb0VSN`), read-only. It re-derived every number from the sealed roots:

- Arm A: Updated 264/273 = 0.96703; all other held-out classes, recency trap 24/24, in-distribution all 1.000, `dev_nll` 2.4925663, tag accuracy 0.99597, `read_events` 7,460, trace `{Emission 12, NotSelected 43}`, cause `{Other 12, QueryEntityTag 1, QueryRelationTag 42}`, params 1,428,816 — all match.
- Arm B: 185/287 = 0.64460, `dev_nll` 2.4337248, tag accuracy 0.99769, `read_events` 7,356, trace `{Emission 76, NotSelected 60, Unavailable 1}`, cause `{Other 75, QueryEntityTag 9, QueryRelationTag 52, WriteValueTag 1}` — all match.
- Baseline `dc-s1`: 43/273 = 0.15751, `dev_nll` 2.3738267 ✓.
- **Rule:** A − baseline = +0.8095 ≥ 0.30 → adopt masking ✓; B does not beat A (−0.3224) ✓; guards: in-distribution PASS, text NLL **FAIL** both arms (correctly reported).
- Receipt 4,742 s, peaks 2.799 / 2.784 GiB, exit 0, no `error.json` — all match.

**Corrections applied (commit `12208df6`):** the evidence JSON now includes the `Absent` in-distribution rows (355/355, 374/374), and the record states explicitly that the pre-registered rule was applied to the aerm-probe's held-out Updated metric, with Lab 1's `prime-route-eval` sieve evaluation still pending.

Class-B conditions hold: sealed roots, non-author review, non-author headline re-read.



---
_Comment/review ids in this file: 5885047499_

