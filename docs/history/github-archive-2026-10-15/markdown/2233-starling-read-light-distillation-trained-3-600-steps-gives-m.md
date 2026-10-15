# #2233 — Starling read: light distillation trained 3,600 steps gives mqar recall 0.780 on 5/5 seeds, no softmax at runtime (KEEP, provisional) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T02:41:46Z
- **Updated:** 2026-10-12T02:43:15Z
- **Closed:** 2026-10-12T02:42:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2233
- **Merged:** 2026-10-12T02:42:49Z
- **Merge commit:** dc07f018ec0338b4c760a1800c45d72c9acd0712
- **Base / Head:** main / UOR-Foundation:claude/starling-s1c
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · **KEEP**: the mechanism target is met, so the count resets (D22) · headline "softmax at runtime: yes" → yes (served model unchanged). References #2032.

Murmuration-read ladder, step 1c: a recall-scored follow-up with 5 seeds per arm ([pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6116521393) with its amendment; FIT). [Record](docs/labs/starling-recall-2026-10-12/README.md).

| Arm | Recall, seeds 1–5 | Mean | Seeds ≥ 0.30 | Row |
|---|---|---:|---:|---|
| W03, base | 0.141, 0.561, 0.439, 0.247, 0.015 | 0.281 | 2/5 | — |
| W01, KL 0.1 | 0.188, 0.910, 0.731, 0.360, 0.009 | 0.440 | 3/5 | 2 |
| **W03L, 3,600 steps** | **0.938, 0.605, 0.950, 0.843, 0.566** | **0.780** | **5/5** | **1, KEEP** |

**Runtime:** the read stays softmax-free. The training aids (helper, surrogate, KL) are training-only, and none is saved or served.

**Caveats, stated in the record:**
- The KEEP is provisional, since the bar is about one SE, so step 2 re-runs the arm.
- The L2 softmax read's 0.617 was measured at 1,800 steps, so the two are not compared at equal budget; step 2 adds L at an equal budget.
- The bench is synthetic mqar at context 512.

**Checks:**
- Docs only, with no code change against main; the claim-wording gate passes.
- The runs used the binary built at `6e64a8734`, whose bench and model code is identical to step 1b's, with `device=cuda` for all 12.
- Every report has `steps_completed` at its target and `stopped_early_at_max_seconds: false`.
- Outputs are in the cloud store (md5 753ae6c7…).

**Also updated:** STATUS, ROADMAP and the M4 row in #2028.

**Would the next step differ if this came out the other way?** Yes. A row-3 outcome would have sent the line to a stability mechanism; this result sends it to ladder step 2 (distance and context, at equal budget with L).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5486117759 — 2026-10-12T02:41:48Z — Casey-allard — COMMENTED

Self-review at exact head 60abf77f3574345fa2256aff58f6198df851a2ba: docs only; numbers match the archived logs (claude-s1c-20261012) and the report JSON completion fields; the rows are scored per the amended pre-registration (bar 0.50, ≥4/5 seeds ≥0.30); caveats (provisional, unequal budget against L, synthetic bench) are stated. APPROVE (self).


---
_Comment/review ids in this file: 5486117759_

