# #1393 — Retain learned integer-code candidates and advance to bounded access

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-25T17:00:11Z
- **Updated:** 2026-10-09T13:17:38Z
- **Closed:** 2026-09-25T17:01:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1393
- **Merged:** 2026-09-25T17:01:57Z
- **Merge commit:** 196f77d5b9ab75ec3b74ea4eb75811831b2eee9d
- **Base / Head:** main / UOR-Foundation:codex/learned-rounding-20260925
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Retain both learned integer-code candidates for D8 rung 3 engineering continuation. Both pass all five original hard-artifact gates. This clears the likelihood-retention barrier that rejected the original and projected discretization recipes, at one paired seed on previously exposed development.

References #973, #820. Their full integrated useful-language and integer-serving acceptance remains open.

## Implementation

- Add legal floor/ceil code choices, tied alpha parameters, differentiable soft values, exact-grid fixed coordinates, explicit global coordinate-mean commitment penalty and hard materialization in `joint_rounding.rs`.
- Integrate alpha gradients into the existing full recurrent language graph without quantizing soft parameters twice; preserve original parents, dyadic scales and fixed interface grids.
- Add training-only eight-batch gradient-norm coefficient initialization, exact alpha/Adam/cursor checkpoint continuation and final packed export/reload in `joint_rounding_campaign.rs`.
- Preserve original model clock 8,348 separately from the 512 new rounding updates. Packed runtime requires no alpha or shadow checkpoint, but still uses F32 numerical emulation.

## Executed paired result

Each arm receives 512 alpha-only updates / 2,097,152 fitted target visits at B16/T256, plus 32,768 normalization visits. Continuous models remain the original retention anchors; this is not a matched additional-training-compute superiority comparison.

| Comparison-tail NLL | Quaternion | Householder pair |
|---|---:|---:|
| Retained continuous | 2.090518499 | 2.064403221 |
| Previous projected packed | 2.150319798 | 2.147928712 |
| New learned hard codes | **2.114226169** | **2.110879800** |
| Hard minus continuous; allowance 0.05 | **+0.023707670** | **+0.046476579** |
| Whole-prefix NoRead penalty; minimum 0.02 | +0.495382663 | +0.497110149 |

Both five-gate vectors are PASS/PASS/PASS/PASS/PASS. First-noun losses against rung 1 are 2/1, within the original maximum 2. Both still lose three previously correct complete answers; gains are separate. Exact completions are 28/32 and 24/32. All ten actual free generations are nonconstant and avoid short-cycle collapse, but remain semantically unreliable. No geometric-advantage, useful-language, Hamiltonian, integer-serving, sparsity or energy claim follows.

Hard code changes are 128,083/126,324. Loaded full-context probability delta is zero, with seeded token/distribution parity. Principal arithmetic verifies 32 sealed roots, 44 binding records, all target rows and source-response changes. Identical actual data hashes bind normalization and all three continuation segments across both arms.

## Local validation and resources

- 42 focused optimized Rust tests passed in 3.59 seconds, plus release build and actual model execution.
- Final formatting, claim-wording and staged diff checks pass. Executed Rust/dependency source is unchanged from `036eabcc86e453f581aa0563e2967cc1f7db96d5`.
- Retained initial failures: a test-only string-key compilation mismatch, then three unprojected test fixtures; corrected before model execution, without relaxing assertions or model criteria.
- RDC ran two concurrent DeepSeek research/implementation sessions and both local Rust fits. No Kimi model or external training hardware.
- Paired fit supervision totals 48.16 minutes. A physical-reserve stop and a configured process-time boundary checkpointed/resumed the same recipe, with no discarded/repeated updates. All unique artifacts and worktrees are preserved. Full chronological costs are charged, including failed builds and delivery tail.
- Compatibility queue acknowledgements execute no tests. Compiler peaks and physical energy are unmeasured.

## Next and documentation

README, canonical ladder, current state, source map, evidence index and training README agree: advance to trained bounded admission and transport, then actual integer execution and useful complete outputs. Count all admission work; full scanning followed by top-k is not a bounded-access claim. No extra rounding/scale sweep.

The large JSON diff preserves the complete training curves, source responses, artifact bindings and principal verdict. The concise entry point is `docs/integration/learned-rounding-result-2026-09-25.md`, with the prospective plan, full output record and resource closeout alongside it.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

