# #10 — feat: proof-carrying semantic routing and hologram compiler redesign (Phase 1-4, Phase 10)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-20T22:46:08Z
- **Updated:** 2026-07-21T22:09:37Z
- **Closed:** 2026-07-21T20:46:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/10
- **Merged:** 2026-07-21T20:46:06Z
- **Merge commit:** abb13700742df00bcc2a9fb5ec26e8aa17645c7b
- **Base / Head:** main / UOR-Foundation:feature/proof-carrying-semantic-routing
- **Reviews / inline review comments:** 1 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This PR implements the following core features:

1. Proof-Carrying Semantic Routing & VSA Algebra (Phase 1-4):
   - Core structures (SemanticRouteReferenceV1, FacetRoute, SemanticSpaceManifestV1, ReasoningPlanV1, SemanticInferenceWitnessV1).
   - 1024-bit VSA binary hypervector operations (bind, unbind, bundle, permute, similarity, expand_atom).
   - Pluggable SemanticGeometry trait in the router crate.
   - Unit test coverage for CBOR types, VSA algebra, and selective backoff.

2. Hologram Compiler Redesign & Oracle Separation (Phase 10):
   - Separated RepresentationSource and BehaviorSource traits from TeacherOracle.
   - Implemented Blake3-seeded deterministic geometric sparse projection.
   - Implemented sampled residual vector quantization (RVQ) centroid training and residual assignment.
   - Scaled token IDs from u16 to u32 across all crates.
   - Formatted binary corpus records to a 32-byte layout with top-3 teacher predictions and weights, with legacy 12-byte corpus parsing support.

## Timeline (5 entries; chronological: conversation comments, reviews, review comments)

### review 4739535513 — 2026-07-20T22:51:11Z — chatgpt-codex-connector[bot] — COMMENTED


### 💡 Codex Review

Here are some automated review suggestions for this pull request.

**Reviewed commit:** `92e8fa381d`
    

<details> <summary>ℹ️ About Codex in GitHub</summary>
<br/>

[Your team has set up Codex to review pull requests in this repo](https://chatgpt.com/codex/cloud/settings/general). Reviews are triggered when you
- Open a pull request for review
- Mark a draft as ready
- Comment "@codex review".

If Codex has suggestions, it will comment; otherwise it will react with 👍.




Codex can also answer questions or update the PR. Try commenting "@codex address that feedback".
            
</details>

### review-comment 3618057473 — 2026-07-20T22:51:11Z — chatgpt-codex-connector[bot]

_on src/tless_uor.rs:386_

**<sub><sub>![P1 Badge](https://img.shields.io/badge/P1-orange?style=flat)</sub></sub>  Keep Tless axis output within the hasher buffer**

When `UorTlessModel::forward` runs, `TlessHasherAndAxis::finalize` allocates `[0u8; R4_FP_MAX]`, and `R4_FP_MAX` is still 32 bytes in `crates/uor-r4-router/src/lib.rs`. Raising the required axis output to 33 bytes makes `TlessAxisImpl::predict` reject that buffer on every full-window hash and fall back to SHA-256, so the grounded fingerprint/trace no longer comes from the transformerless prediction axis even though the server later calls the axis manually for JSON fields.

Useful? React with 👍 / 👎.

### review-comment 3618057477 — 2026-07-20T22:51:11Z — chatgpt-codex-connector[bot]

_on src/tless_uor.rs:490_

**<sub><sub>![P2 Badge](https://img.shields.io/badge/P2-yellow?style=flat)</sub></sub>  Update the output partition offsets for the u32 token**

This new layout writes `token u32 | depth | code | count | ...`, but `TlessPredictOutput::FIELDS` below still describes the old `token u16` layout starting at `(0,2), (2,1), (3,4)`. Any product-field consumer of the grounded output will now split the record at the wrong bytes, e.g. reading depth from the token and shifting every later field, so the UOR witness metadata does not match the bytes produced here.

Useful? React with 👍 / 👎.

### review-comment 3618057479 — 2026-07-20T22:51:11Z — chatgpt-codex-connector[bot]

_on crates/uor-r4-core/src/transformerless/compiler.rs:262_

**<sub><sub>![P2 Badge](https://img.shields.io/badge/P2-yellow?style=flat)</sub></sub>  Avoid appending new records to legacy corpora**

If a previous run left a legacy 12-byte corpus and this function resumes it with a higher target, it keeps the existing append stream but now writes 32-byte records. The metadata count then covers both formats, while `load_corpus_from` only accepts an all-legacy `n * 12` file or an all-new `n * 32` file, so the mixed corpus can never be loaded and the compile command will keep asking the user to rerun.

Useful? React with 👍 / 👎.

### review-comment 3618057481 — 2026-07-20T22:51:11Z — chatgpt-codex-connector[bot]

_on crates/uor-r4-core/src/transformerless/compiler.rs:244_

**<sub><sub>![P2 Badge](https://img.shields.io/badge/P2-yellow?style=flat)</sub></sub>  Handle rounded top-3 weights that exceed 100**

For common probability triples, rounding each normalized top-3 weight can produce a total above 100, such as 34 + 34 + 33 = 101. In debug/test builds `accumulated` is a `u32`, so `100 - accumulated` panics during corpus generation; in release it only works by wrapping before the cast. Compute this delta in a signed type before applying it.

Useful? React with 👍 / 👎.


---
_Comment/review ids in this file: 4739535513, 3618057473, 3618057477, 3618057479, 3618057481_

