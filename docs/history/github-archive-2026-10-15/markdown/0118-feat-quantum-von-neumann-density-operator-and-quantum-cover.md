# #118 — feat(quantum): Von Neumann density operator and quantum cover induction (#108)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T18:29:35Z
- **Updated:** 2026-10-09T14:17:09Z
- **Closed:** 2026-07-23T18:44:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/118
- **Merged:** 2026-07-23T18:44:24Z
- **Merge commit:** 3dd9d58574ddf7855a582da5b3143c955b1a2867
- **Base / Head:** main / UOR-Foundation:issue-108-quantum-cover
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #108

## What

`crates/uor-r4-core/src/transformerless/quantum_cover.rs` — Von Neumann maximum-entropy density operator and quantum cover induction per `docs/r4_furey_quantum_geometric_plan.md` §Phase 5.

- `DensityOperator`: diagonal ρ (eigenvalues = normalized distribution), `max_entropy(n)` = (1/n)I, `trace()`, `von_neumann_entropy()` = −Tr(ρ ln ρ) in nats, `max_entropy_bound(n)` = ln n, `equipartition()` = S/S_max.
- `quantum_entropy_gain(observations, members, children)`: the entropy-gain criterion for cover induction, S(ρ_parent) − Σ w_c·S(ρ_c) over next-token density operators — the nats analogue of `cover::entropy_reduction`.
- `QuantumCoverConfig`: accept floor, default = the compiler's classical floor (0.25 bits) converted to nats — both criteria accept at the same information threshold.
- `quantize_residuals(&[f32]) -> Vec<ScoreQ>`: emission residual quantization into Q16.16 integer tables, delegating to `ScoreQ::from_logprob` (saturating, NaN→0).

Honesty note (in the module doc): for the diagonal operators used here, S(ρ) coincides with Shannon entropy in nats — the value is the operator framing, the equipartition bound, and the nats criterion, not a different number. The module is compiler-side and opt-in: no pinned compiler default, artifact, or Gate C anchor changes.

## Gate C 4-way table (fixture, unchanged — this PR touches no scoring path)

| Rule | top-1 | bits/token |
| :--- | ---: | ---: |
| legacy_sum | 0.2863% | 76.9643 |
| rule1_chain | 1.2185% | 71.5219 |
| rule12_precedence | 31.7086% | 9.8612 |
| tla3_baseline | 31.7086% | 11.8781 |

The DoD's D3-partition numbers of record remain #75's published evaluation; a fresh D3 run is out of scope here (the on-disk `.uor-models` store predates the u32 token migration and needs a full recompile; this change cannot move Gate C either way).

## Tests (6, all green)

- S((1/n)I) = ln n to 1e-6; pure state → 0; two-point distribution vs hand-computed
- invalid distributions are typed errors, not panics
- informative split gains ln 2 nats and is accepted; noise split gains ~0 and is rejected
- quantization matches `ScoreQ::from_logprob` element-wise, round-trip within Q16.16 resolution

## Verification

`cargo test --workspace --offline` green; `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean; `cargo fmt --check` clean.

Refs #103 (tracker), #75 (D3 evaluation of record)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

