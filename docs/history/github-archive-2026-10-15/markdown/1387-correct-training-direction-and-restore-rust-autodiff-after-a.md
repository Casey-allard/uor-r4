# #1387 — Correct training direction and restore Rust autodiff after A2–A4

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T19:11:11Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-09-24T21:41:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1387
- **Merged:** 2026-09-24T21:41:14Z
- **Merge commit:** 942645b264f73ec49507cffd7c2f4cbd95de3aa2
- **Base / Head:** main / UOR-Foundation:codex/integrated-attention-a2-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Acts on the owner's supplied whole-project stuck-point review and request to make warranted corrections, then complete the already authorized protected merge.

**The central criticism is supported.** A2–A4 improve pieces of native addressing, but the local-credit training succession still fails complete language behavior. D8 parks further selector tuning as the primary learning strategy and adopts a persistent reference → jointly learned native recurrent-memory student → explicit discretization → bounded admission → integer-export ladder. D0-b and D4–D6, exact memory, transformerless target serving, and the eventual parameter-sparsity goal remain intact.

This PR retains the A2/A3 work already reviewed here, adds the completed matched A4 negative result, and delivers an actually executed Rust autodiff foundation for the corrected route. No native model is promoted.

## Latest A4 result

Four eight-epoch runs each consume 1,124,776 new token updates. Both A4 arms learn one bounded integer candidate-or-NoRead action; matched continuation controls receive the same repeated language-head adaptation.

| Measure | C120 A4 | C120 control | 2I A4 | 2I control |
|---|---:|---:|---:|---:|
| New first-decision source admission | 17/24 | 17/24 | 17/24 | 17/24 |
| Correct source selected | 2/24 | 0/24 | 2/24 | 0/24 |
| Complete correct read-enabled answers | 0/12 | 0/12 | 0/12 | 0/12 |
| Development bits/token, read enabled | 6.839477 | 6.799839 | 6.798995 | 6.872785 |
| Final loaded fit sources selected | 28/192 | 0/192 | 24/192 | 0/192 |

The fit denominator includes 167 admitted sources per arm. Every eligible local provenance update succeeds immediately, yet most source decisions are lost by export. The dedicated address bank is fixed but shared State rows and causal trajectories change actual fine codes. The original sealed report's broader freeze wording is explicitly corrected without altering its evidence.

All 112 generation rows and eight complete files per arm reproduce in independent replays. Actual outputs remain fragmentary. This supports changing the training method; it does not isolate one cause or establish geometric disadvantage.

## Implemented correction

- Added offline-only `crates/uor-r4-training`, with Candle core/nn pinned to 0.9.2 and CPU/Metal support. No serving crate depends on it.
- Loads the actual retained #1017 F32 safetensors checkpoint safely; validates weights SHA and all 56 tensor names/shapes and architecture.
- Implements differentiable RMSNorm, HF half-rotation RoPE, causal softmax attention, residuals, SwiGLU and tied output. Next-token language loss reaches Q/K/V through composed operations with backward support.
- Source-bound optimized build `e4f7e9d9` executed on both actual devices. The unchanged reference oracle uses Apple Accelerate CPU arithmetic.
- Pinned evaluator inputs and thresholds before running. Independent review verifies emitted input hashes, token window, actual device, gradients and finite differences against that manifest. The manifest is checked independently; it is not claimed immutable or automatically enforced by the CLI.

| Integrity check | CPU | Metal |
|---|---:|---:|
| Compared logits | 131,072 | 131,072 |
| Maximum absolute logit difference | 0.000015259 | 0.000016212 |
| Top-one matches | 32/32 | 32/32 |
| Finite nonzero parameter-gradient tensors | 56/56 | 56/56 |
| Finite nonzero Q/K/V-gradient tensors | 18/18 | 18/18 |
| Selected finite differences passing | 3/3 | 3/3 |
| Optimizer steps | 0 | 0 |

These are 32-position reference integrity checks, not a new language-quality result, full baseline table, native student or discrete gradient bridge. The approximate 1.358 training-prefix loss must not be compared with historical #1017 test NLL. CPU/Metal process times of 1.39/12.88 seconds include loading, oracle comparison, backward work and finite differences; they are not training-throughput or GPU-active measurements.

## Project and research reconciliation

The attached review is assessed against current code, history and primary literature. It correctly identifies missing joint language credit, inadequate scale for language conclusions and a repeating micro-stage process. Its A1 diagnosis predates A2–A4; Rust policy already permits autodiff and float/matmul training; #1017 had later grounding/LoRA work; and historical revealed tests are regression data, not fresh holdouts.

The assessment preserves mathematical distinctions among geometric labels, noncommutative transport, 600-cell codebook quality and exact algebraic arithmetic. LEMA/Transformer-VQ support a soft-to-hard bridge, not a universal impossibility theorem for discrete learning. A learned read energy is not automatically a physical Hamiltonian. Exact event memory remains necessary.

D8 updates the canonical plan, current state, README, execution policy and team protocol. The old 4,324-line current state is preserved byte-for-byte in a dated archive; routine recovery now starts from the concise live page. The plan specifies a recurrent language/read/update computation graph and a baseline-table entry gate, rather than another local selector adjustment.

## Executed verification and resources

- 33 focused integrated-attention tests passed; no blanket full-suite claim.
- Optimized source-bound A4 build, four fits and four independent replays completed; negative artifacts preserved.
- Optimized offline Rust/Candle build and actual CPU/Metal numerical-gradient checks passed; independent source and evidence review found no material defect.
- `cargo fmt --check`, claim-wording check and `git diff --check` passed.
- No existing Cargo.lock package versions removed or upgraded; new dependencies only. Default workspace members unchanged.
- A4 new exposure: 4,499,104 token updates; joint fit time 185.945 seconds. Reference integrity uses zero optimizer steps.
- Full engineering cycle at closeout: 3,440,411 ms, plus 300,000 ms booked delivery reserve. Cumulative ledger 479,596,977 / 484,200,000 ms; necessary extensions were recorded prospectively.
- Physical free at closeout: 27,260,551,168 bytes, retaining the protected reserve and stop margin. No unique-artifact deletion, paid compute or owner-checkout changes beyond the shared ledger.
- Required queue status names remain compatibility acknowledgements, not executed tests.

## Evidence and remaining work

- `docs/integration/stuck-point-review-response-2026-09-24.md`
- `docs/integration/project-track.md` and `docs/integration/DECISIONS.md` D8
- `docs/integration/current-state.md`
- `docs/integration/reference-evaluator-v1.json`
- `docs/evidence/reference-autodiff-integrity-2026-09-24.json`
- `docs/integration/integrated-attention-a4-result-2026-09-24.md`
- `docs/evidence/integrated-attention-a4-result-2026-09-24.json`
- `docs/evidence/stuck-point-review-closeout-2026-09-24.json`

Next within rung 0: the same-token reference/n-gram/cache baseline table and actual reference generation replay. Then implement the declared recurrent learning graph, measure training throughput, and run a meaningful, matched natural-text campaign before hardening. Student learning, geometry attribution, final export, persistent-session saturated-page restore, D5 full-path access and physical energy remain open.

References #973 and #820. Neither complete acceptance issue is closed.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

