# #1906 — Price the transport-snap fix: an order of magnitude of training throughput (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T13:15:56Z
- **Updated:** 2026-10-09T13:22:22Z
- **Closed:** 2026-10-08T13:17:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1906
- **Merged:** 2026-10-08T13:17:37Z
- **Merge commit:** f925d2e4e4e3ce5eec38a9ef4c14411327c17343
- **Base / Head:** main / UOR-Foundation:deepseek/snap-cost
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this prices

Round 24 proposed `transport_snap=icosian` as the fix for the largest remaining served term — **0.03578 BPB, 64.8 % of the whole served gap** — reasoning that a model trained in the representation it is served in shouldn't pay to be snapped into it.

Same 19.9 M stack, same corpus, same 12,207 steps, same hyperparameters, same pod, same binary. **One flag different:**

| arm | GPU utilisation | first eval (step 4,000) | checkpoints |
|---|---:|---|---|
| continuous transport | **100 %** | reached in **~150 s** | written |
| `transport_snap=icosian` | **15–25 %** | **not reached in ~25 min** | none |

Snapped arm process evidence: **336 % CPU, 38–40 min of CPU time inside 11–12 min of wall**, GPU holding 4,498 MiB but idle three-quarters of the time. The snap sits on the CPU critical path **every step**.

## The trade, stated honestly

For a project whose stated objective includes **lower energy and wasted compute**, this is not a free flag. The real decision is **0.03578 BPB of served fidelity against roughly an order of magnitude of training throughput** — and it can't be weighed without the fidelity number, which the run has not produced.

I'm recording the measurement and the trade rather than picking a side on half the evidence.

## The cheap form of the idea, if the gain is real

The cost is in *training* with the snap, not *serving* it. If the gain holds, the cheap way to buy it is a **short snapped fine-tune from a continuously trained checkpoint** — enough steps to move the weights into the snapped basin, instead of paying the slow path for all 12,207 steps. Testable: take the round-16 checkpoint, fine-tune with `transport_snap=icosian` for a few hundred steps, then `snap-evaluate`. If `snapped_nll` collapses toward `fused_nll` quickly, the flag's cost is paid **once and briefly**.

**No claim that this works** — it's recorded as the cheapest next test.

Scope: one model, one corpus, one pod, two runs differing only in the flag; throughput figures are process/GPU observations, not a benchmark harness, and "not reached" entries are recorded as **absences** rather than estimates. Evidence: `docs/evidence/transport_snap_cost_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

