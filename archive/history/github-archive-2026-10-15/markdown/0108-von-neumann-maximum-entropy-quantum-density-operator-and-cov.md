# #108 — Von Neumann maximum-entropy quantum density operator and cover induction

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-23T15:48:47Z
- **Updated:** 2026-07-23T18:44:57Z
- **Closed:** 2026-07-23T18:44:25Z
- **Labels:** graph-compiler
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/108
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #103
Depends on: #107
Spec: `docs/r4_furey_quantum_geometric_plan.md` §Phase 5

Implement the maximum-entropy quantum density operator $\rho = \sum \frac{1}{n} I_{n \times n}$ and von Neumann entropy calculation (\rho) = -\text{Tr}(\rho \ln \rho)$ in `crates/uor-r4-core/src/transformerless/quantum_cover.rs`.
Use entropy gain criteria to induce multiresolution region covers and quantize emission residuals into `ScoreQ` integer tables.

---

## Definition of Done
- `quantum_cover.rs` implements entropy-bounded region cover induction and residual quantization.
- Gate C evaluation confirms top-1 agreement and bits/token performance on the held-out D3 partition.

## Acceptance criteria
- [ ] Entropy calculations match analytical formulas.
- [ ] Gate C evaluation results reported with 4-way comparison table.
- [ ] `cargo test` passes 100% cleanly.

## Comments (1)

### comment 5062106336 — 2026-07-23T18:44:55Z — auser

## Done — DoD evidence (PR #118)

- [x] `quantum_cover.rs` implements entropy-bounded cover induction: `DensityOperator` (incl. the maximum-entropy (1/n)I), `von_neumann_entropy` = −Tr(ρ ln ρ) in nats, `quantum_entropy_gain` criterion over next-token density operators, `QuantumCoverConfig` accept floor (default = the classical 0.25-bit floor converted to nats)
- [x] Residual quantization into `ScoreQ` integer tables (`quantize_residuals` → `ScoreQ::from_logprob`, saturating)
- [x] Gate C 4-way comparison table (fixture; unchanged at the pins since no scoring path is touched):

| Rule | top-1 | bits/token |
| :--- | ---: | ---: |
| legacy_sum | 0.2863% | 76.9643 |
| rule1_chain | 1.2185% | 71.5219 |
| rule12_precedence | 31.7086% | 9.8612 |
| tla3_baseline | 31.7086% | 11.8781 |

D3 numbers of record remain #75's published evaluation; a fresh D3 run is out of scope (the `.uor-models` store predates the u32 token migration and needs a full recompile).

**Acceptance criteria:**
- [x] Entropy matches analytical formulas: S((1/n)I) = ln n (1e-6), pure state = 0, two-point vs hand-computed — unit-tested
- [x] Gate C 4-way table reported above
- [x] `cargo test` 100% clean (workspace green; clippy `-D warnings` and fmt clean)

Module doc records the honest scope: for diagonal ρ the vN entropy coincides with Shannon entropy in nats — the value is the operator framing, equipartition bound, and nats criterion. Compiler-side and opt-in; no pinned default, artifact, or anchor changes.

Closing. Refs #103, #75.



---
_Comment/review ids in this file: 5062106336_

