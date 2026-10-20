# #278 — certify C-stage (R4G1 eval) is O(suffix x node_count x edge-decode) per position — looks like a hang, gates the queue; add probe time budget now, emit index later

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-07-29T13:44:15Z
- **Updated:** 2026-07-29T21:28:12Z
- **Closed:** 2026-07-29T21:28:12Z
- **Labels:** bug
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/278
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
`r4 certify` appears to hang after printing the "B bit-prefix" row. It is not a deadlock: a 10s `sample` of the live process (8182 samples, two independent runs reproduced it — one ran 15h05m at 100% CPU with zero further output before being killed) shows all time inside `R4G1Runtime::predict_distribution`, ~94% in `check_node_emits` → `GraphView::edge` → `decode_entry`/`decode_edge`.

### Root cause
Per predicted position, `predict_distribution` (crates/uor-r4-graph-runtime/src/engine.rs) iterates suffix lengths 10..1 and for each one scans **every node in the graph** through `check_node_emits`, which re-decodes edge records from the serialized container on every access — no emit index, no decode cache. Cost ≈ 10 × node_count × per-node edge decode, per position.

The #232 readiness probe exists precisely to avoid "spending hours on a full evaluation", but the probe itself pushes `PROBE_POSITIONS = 64` positions through this same path. Observed rate on the current artifact (~406k-key store): the probe did not finish 64 positions in 15 hours (≥14 min/position); the full held-out evaluation behind it is weeks. The stage between the "B bit-prefix" println and the C row also emits no progress output, so it is indistinguishable from a hang from the outside — this burned two working sessions on environment theories (sleep/timeouts) before sampling.

### Immediate mitigation (PR attached)
Certifier-side only, no runtime-kernel change: give the readiness probe a wall-clock budget (default 120s, `R4_CERTIFY_R4G1_BUDGET_SECS` to override). A probe that cannot finish inside the budget classifies as a new explicit outcome and the C row prints as SKIPPED with the measured per-position rate — the same absent-measurement discipline as the existing `NoScoredEmission`/`ConstantPrediction` skips (#232). The full evaluation, when it does run, gets per-position progress lines and the same budget check so a future stall is visible in minutes, not hours. A/B rows, op census, equality witness, and compression witnesses are unaffected and complete in ~5 minutes — certify stops gating the queue on a computationally infeasible row.

### Real fix (follow-up, this issue stays open for it)
Make the C evaluation feasible instead of skipped: build an emit-token → node index once per runtime parse (certifier-side or a precomputed section), and/or cache decoded node/edge records instead of re-decoding per access. DoD: the probe completes in seconds and the full held-out C row is measured, not skipped, on the current artifact size.

Stack sample retained at /tmp/certify_sample.txt (macOS `sample` output).


## Comments (2)

### comment 5119492653 — 2026-07-29T14:55:09Z — Casey-allard

Measured correction on magnitude (from the parallel #244 session, same machine, same D3 artifacts): the structural diagnosis here is right — `check_node_emits` → per-access edge decode with no emit index is the hot path — but the per-position rate is **~1.2–1.6 s/position, not ≥14 min/position**:

- A bounded certify run this morning completed the **64-position probe AND a 1,002-position deterministic stride-sampled C evaluation in ~25 minutes** (row posted on #244: top1 0.0% / agreement 0.0%, probe 64/64 scored).
- This PR's own verification log agrees: `256/30036 in 402s` ≈ 1.57 s/position.
- The 15h05m run that was killed was therefore almost certainly ~95% through the **full 30,036-position eval** (~13h at that rate), not stuck in the probe. The "full eval is weeks" estimate should read **~13h** — infeasible for the queue either way, so every mitigation here stands; only the magnitude needed the correction.

Two follow-ups this suggests:

1. **Prefer a stride subsample over discarding partials**: a budget-truncated prefix is a biased sample (earliest held-out positions only), but a deterministic stride subsample of ~1,000 positions fits comfortably inside the 600s budget and records a *real* row instead of a skip — that's how the 0.0% measurement on #244 was produced. PR forthcoming that layers this onto the budget machinery here.
2. **Priority note**: per the #244 amendment and #280, the graph readout currently predicts 0.0% against 28.2% for the same store through the store path — so the emit-index perf fix, when it comes, will be measuring a broken readout until #280 is resolved. #280 first, then this issue's real fix.

### comment 5123548605 — 2026-07-29T21:28:11Z — Casey-allard

Closing as **overtaken by the #280 decision** (PR #288, merge 89d74d9): the emit-token→node index and decode caching proposed here targeted `convert_r4g1`-produced artifacts — a path since diagnosed as never functional (no per-node emissions to index; the readout decoded container bytes at the wrong stride) and demoted to container-format demo status. Indexing it would make a non-functional readout faster.

What this issue actually delivered stays delivered: the budget/skip machinery (PR #279) and its env contract live on in the new serving-surface C row, and the diagnostic record here (the O(suffix × node_count × edge-decode) profile, /tmp/certify_sample.txt methodology) remains the reference for why certify stalls must always be visible, budgeted, and recorded. If a future emission wiring lands per the #243/#248 architecture, per-parse indexes belong in that design from day one — link back here.


---
_Comment/review ids in this file: 5119492653, 5123548605_

