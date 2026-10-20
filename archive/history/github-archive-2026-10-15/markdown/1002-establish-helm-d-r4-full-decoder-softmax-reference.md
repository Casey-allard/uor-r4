# #1002 — Establish HELM-D R4 full-decoder softmax reference

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-29T17:15:10Z
- **Updated:** 2026-10-09T14:17:20Z
- **Closed:** 2026-08-29T17:16:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1002
- **Merged:** 2026-08-29T17:16:26Z
- **Merge commit:** f1ab400827bd226f6af97138a63628cfd49af115
- **Base / Head:** main / UOR-Foundation:issue-973-helm-d-r4-softmax
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Establishes the first positive full-language-decoder attention reference for the active #973 programme: ordinary full-prefix causal softmax is expressed through exact cumulative R4/Spin/H4 local frames while retaining learned Q/K/V, RoPE, GQA, value aggregation, Wo, residual/FFN blocks, final norm, and the LM head.

The bounded held-out run passes numerical and behavioral parity. The source-frame-permuted equal-work control breaks the function, so the transport seam is live. This authorizes the next intrinsic R4 distance/centroid rung; it does not claim geometric advantage or transformerless serving.

## Evidence

- pinned official HELM-D architectural source at `7501deca8f413848bfef804be64ce874b72a3cd7`, with MIT provenance and a golden Lorentz attention fixture
- frozen SmolLM2 donor CID `blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5`
- raw SimpleWiki corpus bytes verified as `blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf`
- result CID `blake3:05eaad210198fbe39a0645c25b0c890c55d5f3d3dd8a1710472e976a637e2a07`
- 196,608 full-vocabulary logits; max/mean donor-R4 delta `1.049041748046875e-5` / `2.2742100540540378e-6`
- teacher-forced top-1 3/3 equal; donor/R4 decoded continuation `, and`
- donor and coherent-R4 replay exact; zero future reads
- 103,680 encoded R4 blocks; 43,200 key and 43,200 value block transports
- permutation control executed 77,760 changed source-frame actions, decoded `[[`, and shifted logits by 23.0844 maximum
- exact-math executor used 8 available workers

## Verification

- focused model-source transport tests: 2 passed
- focused HELM-D/R4 unit tests: 3 passed
- byte-final ignored release decision probe: passed in 70.56 seconds
- focused clippy for changed model-source/core paths: passed after isolating pre-existing repository-wide Rust 1.97 lint debt
- wasm32 dashboard facade check: passed
- rustfmt, claim-wording, and diff checks: passed
- independent implementation review: no remaining blockers

## Project direction

Updates repository documentation, ADR-0005, roadmap, terminology, historical successor notes, #820, #954, and #973 to the evidence-backed sequence: HELM-D provenance and ordinary donor -> gauge-equivalent R4/Spin softmax parity (now PASS) -> intrinsic R4 distance plus geometric centroid (active) -> multi-resonance replacement -> bounded recurrence -> exact lowering and scale.

Upstream HELM-D checkpoint parity, intrinsic R4 advantage, softmax removal, recurrence, transformerless serving, scale, correctness, and product readiness remain NOT_RUN or unclaimed.

References #973

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

