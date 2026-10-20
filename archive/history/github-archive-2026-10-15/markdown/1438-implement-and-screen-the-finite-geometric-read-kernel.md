# #1438 — Implement and screen the finite geometric read kernel

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T19:13:50Z
- **Updated:** 2026-10-09T13:17:29Z
- **Closed:** 2026-09-27T21:01:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1438
- **Merged:** 2026-09-27T21:01:20Z
- **Merge commit:** dccef74b4b73460756971a184e8b9fdbfd6096b5
- **Base / Head:** main / UOR-Foundation:codex/geometric-read-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements and screens the fourth lab's designed first operator — an optional, absent-by-default **finite geometric read kernel** (a learned signed relative-group score over the 120-element binary icosahedral group `2I`) on the mainline joint learner — and records its first matched screen. References #973 under #820.

Frozen contract and decision rule: `docs/integration/geometric-read-plan-2026-09-27.md`; design source: `docs/integration/fourth-lab-geometric-attention-2026-09-26.md`.

## What shipped

- `crates/uor-r4-training/src/geometric_read.rs`: versioned kernel config; 16 unit-coded lanes; nearest signed `2I`-root quantization (antipodes never folded, lowest-index ties, zero lane → identity); exact composition via the core `group_table()`; straight-through hard-forward/smooth-backward Hamilton surrogate; per-lane nonlinear 4→8→1 score (784 parameters); detached per-event key-code/coordinate caches; per-lane code/relation usage histogram. **Absent by default**: with the kernel off, parameter sets, graph, contracts and artifacts are byte-identical.
- Minimal adapter in `joint_model.rs` (field, `new_with_geometric_read`, parameter inventory, `read_scores` branch, save/load), campaign/checkpoint config, optimizer fresh-parameter loading, example wiring, and 13 focused tests.
- Screen: arm A (kernel enabled) versus arm B (same binary, kernel disabled), both resuming the accepted step-15,672 quaternion parent for 1,024 updates / 4,194,304 targets, B16/T256/full256/shard2, seed 240924, with the frozen evaluator and prose criteria.

## Result: HARM (predeclared)

| Endpoint | Parent | B (kernel off) | A (kernel on) |
|---|---|---|---|
| Comparison-tail Read NLL (nats/target) | 1.996490474 | 1.984752805 | 2.004471395 (**+0.019719**) |
| Complete source answers (32 rows) | 21 | 27 | **17** (**−10**; 10 unique losses, 0 unique gains) |
| Five-prompt sampled prose | 0/5 | 0/5 | 0/5 |

Numerical and instrument guardrails hold: NoRead penalty ≥ 0.02 nats in both arms; `ReadNLL(A) ≤ parent + 0.03`; no short-cycle stops; hard-path usage non-collapsed (120 distinct key codes and 120 distinct relation codes per lane; 99.25% non-identity; 784 parameters; cumulative over the fit). The **predeclared 1.5× cost gate failed** (1.84× paired same-session) and is carried as a recorded budgeted exception; under the plan's heading a guardrail failure is itself a HARM condition, so **HARM** holds independently of the row criteria (`dComplete ≤ −4`, unique losses ≥ 4). The mechanism is parked at this scope with no automatic repair-fit and no promotion. Narrowed scope: this rejects the finite relation score as parameterized here, including its 784 added per-lane parameters and unit coding, at this dose and representation; the capacity/normalization attribution control is unrun.

## Instrument checks

- The kernel-off path with the new binary reproduces the retained parent evaluation token-for-token (witness `evaluate-quaternion-baseline-invariance-8a6d2-read`; an earlier attempt `…-8a6d-read` failed as an unbound-source build and is recorded as a failed attempt).
- An accidental 341-update kernel-off run replays the historical `plain-16696` learning curve with zero NLL mismatches; arm B reproduces that historical evaluation exactly (identical `targets_sha256`; anchor 1.984753).
- The predeclared cost optimization (per-event key-code caching + vectorized per-lane score) reduced kernel per-step cost from 15.300/8.742/12.879/10.247 s to 6.081/5.443/6.603/9.808 s; the paired same-session ratio is 1.84× (kernel-off 3.796 s/step under concurrent cross-lab load). The attribution of the residual cost to the smooth Hamilton composition comes from implementer profiler sampling, not an independent measurement.
- The conditional norm-controlled attribution arm C had its predeclared trigger fire but is **deferred**: at the decision point the ledger read 712,153,247 / 744,000,000 ms (up 13.3M ms from the freeze balance through concurrent cross-lab consumption); the milestone projects ~730M after charging, and arm C's ~2.5 h including implementation would project ~739M against the predeclared 735M stop margin and the 744M ceiling. The arithmetic is recorded in the evidence rather than left implicit.

## Evidence, limits and claim scope

Evidence `docs/evidence/geometric-read-2026-09-27.json` (+ campaign copies); result `docs/integration/geometric-read-result-2026-09-27.md`; `docs/integration/current-state.md` updated. All endpoints are exposed development data with a single seed and parent and no fresh holdout; concurrent cross-lab load inflated absolute wall times; the 32-row authored panel is small and dose-sensitive, so the row-based harm may overstate a per-seed effect, and the strongest caveat is that the harm may arise from added capacity/optimizer interaction rather than finite-relation coding (arm C unrun). This rejects this parameterization at this dose and representation — not geometric readers as a family. Nothing is promoted and no serving path changed.

References #973, #820 (partial result; does not close either).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

