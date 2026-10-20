# #1499 — Lab 2 (OpenCode): Arm C result — triggers fixed, binding lost, NLL over bound

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T09:19:53Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-09-29T10:14:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1499
- **Merged:** 2026-09-29T10:14:10Z
- **Merge commit:** 05e766d22e88588d3202aa0528e4cc801042a4a7
- **Base / Head:** main / UOR-Foundation:lab/opencode/arm-c-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 2 (OpenCode). Class B** (measured result). References #973 and #820.

Records Arm C, pre-registered in [#973 comment 5884957504](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5884957504).

## Headline

| Prediction | Threshold | Measured | Verdict |
|---|---|---|---|
| Held-out Updated | ≥ 0.90 | **0.553** (151/273) | **FAIL** |
| Text triggers / 1,000 | ≤ 5 | **0.0** | **PASS** |
| Text NLL vs baseline 2.3738 | ≤ +0.05 | +0.083 | **FAIL** |
| Guard: in-distribution every class | ≥ 0.98 | 1.000 | **PASS** |

Both pre-registered decision branches fire: the prose trigger supervision also carries the binding problem (all 411 held-out failures are `QueryRelationTag`), and the NLL cost persists at zero triggers, so the next fix is the pre-registered structural one — gating address-driven reads to user turns. Nothing is promoted.

## Files and evidence

- `docs/integration/arm-c-result-2026-09-29.md` — the record.
- `docs/evidence/arm-c-2026-09-29.json` — assembled from the sealed root, the receipt and the baseline root.
- Checkpoint for Lab 1's `prime-route-eval`/`memory-replies`: `/Volumes/UOR-Workspace/uor-r4-lab/opencode-arm-c/checkpoints/aerm-s1`.

## Checks

- `python3 scripts/check_claim_wording.py` → pass.
- Root `opencode-arm-c/arm-c` sealed, exit 0; receipt `arm-c.supervision.json`; executable `cfe5c6bd…`; 2,709 s wall; peak 2.82 GiB.
- The option split under test merged in #1498 (`stack_aerm` tests pass).

**Merge holds for Lab 1's class-B re-read**, per the standing rule ("#1489 was re-read only after it merged"). A non-author review is also requested.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

