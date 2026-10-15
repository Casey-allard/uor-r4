# #1377 — Adopt the 2026-09-24 direction decision: goal decomposition, terminal sparsity invariant, and the M1 diagnostic gate (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T07:54:00Z
- **Updated:** 2026-10-09T13:17:15Z
- **Closed:** 2026-09-24T07:55:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1377
- **Merged:** 2026-09-24T07:55:03Z
- **Merge commit:** f6f0d3ace981f13c7b9c664e4df57a9a4fdfdeea
- **Base / Head:** main / UOR-Foundation:codex/direction-adoption-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820.

Adopts, on owner instruction (2026-09-24), the recommendations of an independent three-reviewer reassessment
(architecture/K3, mathematics, computer science) of
[`repo-review-direction-2026-09-23.md`](docs/integration/repo-review-direction-2026-09-23.md), whose five §11 owner
decisions were still pending. Docs-only; no artifact, no model run, no served-arithmetic change.

## Decisions applied

- **Goal decomposition (D4).** **Goal S** (multiplier-free serving) is kept and is theorem-backed for the permitted
  operator class. **Goal R** (geometric predictive advantage) is re-scoped from an assumption to a **gated
  hypothesis** and a candidate **sparse-access structure** — retained by default only where already load-bearing
  (identity, addressing, version authority, serialization), and admitted to a causal serving role only by beating an
  information- and compute-matched ordinary control. No geometric mechanism has done so on any tested task.
- **Terminal sparsity invariant (D5).** D0-a's per-token parameter-sparsity test survives D0-b as the **end-state**
  contract; the current served path is recorded as dense/non-compliant with that end state (611,814
  inspections/step vs 56,457 nonzero). This makes Goal R the measurable sparse-access/routing contest.
- **Objective (D6).** The target objective becomes a **long-range information probe** where the order-2 count prior
  is at chance by construction; the count prior is retained as a standing local control, not a target.
- **Staged plan adopted with amendments** (Stage 0 record correction → Stage 1 frozen-state adjudication → Stage 2
  gate × capacity, crossed with corpus → Stage 3 cross-window state + long-range objective → Stage 4
  bounded-integer/LUT attention → Stage 5 LUT + scale + whole-path M1), plus the reviewers' added
  **memory-augmented** track. Full record: [`direction-decision-2026-09-24.md`](docs/integration/direction-decision-2026-09-24.md);
  owner entries D4–D6 in [`DECISIONS.md`](docs/integration/DECISIONS.md).

## Record corrections (Stage 0)

- `model-direction-2026-09.md`: the stale "`prev` at 2.1 %, second-order context absent" claim is replaced by the
  merged depth-observer result (97.84 / 74.86 / 18.69 / 0.60 % at lags 1–4), and the rank wording is split by scope
  (input width 147; effective per-position rank ≤ 64 for prose).
- `README.md`: states that the current best natural-text artifact contains **no geometry**, and formalises the
  disclaimer that UOR-R4 is **not** a "world model".
- `project-track.md`: adds the adopted directive and ordered milestones; subordinates the prior
  "reduce the prose cost of the Stop fix" to the M1 gate.

## Recommended next step — the "escape from bigram class" diagnostic gate (M1)

Plan: [`escape-bigram-class-plan-2026-09-24.md`](docs/integration/escape-bigram-class-plan-2026-09-24.md).

- **Part A — frozen-state adjudication (cheap, no retraining):** arms (i) linear float, (ii) rank-K bilinear,
  (iii) 2-layer MLP, (iv) transport-decode depth ladder (full + restricted), **(v) count-augmented linear as the
  decision arm**, on document-held-out recorded states. Pre-declared rule: if (v)−(i)≈0 the state's local info is
  subsumed by the count table and the lever is the state/horizon, not the readout.
- **Part B — one gate × capacity arm on a synthetic induction panel** (V=64, 512 query positions, count prior
  verified at chance): arms (a) current 64-dim no gate, (b) + hard compare/select gate, (c) 256-dim, (d) 256+gate,
  (e) matched ordinary control; two seeds. **Accept** if an arm beats chance and the count control by ≥2 bits →
  the recurrence class can represent a longer dependency and capacity/context was binding. **Reject** if all arms
  are at chance → escalate to the owner (hybrid integer attention = goal change, or memory-front-end
  repositioning).

## Verification

`python3 scripts/check_claim_wording.py` passed. Docs-only: no code path, build or model run changed. Ledger charged
`+400,000 ms` (decision + review preparation; no local training in this delivery) → `452,439,320 / 455,500,000 ms`.

## Limits

The decisions are governance and direction, not measurements. No capability, energy, reasoning, coding or
geometric-superiority claim follows. The M1 milestone must be started under a recorded prospective resource
projection (the ledger is ≈ 99 % consumed).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

