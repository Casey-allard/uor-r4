# #259 — Route stored geometric coordinates' provenance through uor-addr — auditable resonance retrieval (#246 companion)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T11:50:58Z
- **Updated:** 2026-08-02T00:08:04Z
- **Closed:** 2026-08-02T00:08:04Z
- **Labels:** enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/259
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
Companion to #246 (spectral projection re-port). When stored geometric coordinates become content-bearing (#245, now merged) and spectrally projected (#246), route their **provenance through uor-addr**: each indexed sentence's coordinate carries a certificate (κ of the source text + the projection parameters that produced the coordinate), so resonance retrieval is not just alive but **auditable** — any retrieved memory can be traced to the exact text and transform that produced it.

### Why this matters beyond r4
This is the step where r4 stops using UOR *around* inference (artifact pinning, response envelopes) and starts using it *in the loop* — the reference-implementation story the standards program needs. It also gives the memory-lift A/B harness (#255) a provenance axis: arms can be verified to contain what they claim.

### Sketch
- `CorpusItem` gains a provenance field: κ-label of the source sentence bytes (canonical text axis) + κ of the projection config (zeta design matrix version, window count, vocab-state digest — note vectors are vocabulary-relative, recorded in #252's tests).
- Retrieval responses can surface the certificate chain on request.
- Verification: recompute κ from stored sentence text and compare — a cheap integrity check over the whole angular store.

### DoD
Provenance fields populated at index time via the uor-addr crate (not raw blake3 — see the attestation conformance issue); a verify pass over an indexed corpus; one retrieval surface exposing the chain. Depends on #246 for the projection-config half; the text-κ half can land independently.

*From Maura's suggestion on the optional third step of the geometric revival.*


## Comments (3)

### comment 5147966482 — 2026-07-31T22:23:19Z — auser

Implemented and published as PR #314: https://github.com/UOR-Foundation/uor-r4/pull/314

The router now records source-text, projection-configuration, and vocabulary-state uor-addr κ addresses for indexed geometric coordinates; verification recomputes the chain and resonance retrieval exposes it. Added focused tests for normal and content-free indexing plus documentation.

Validation passed: fmt, diff check, both graph-format no-std checks, workspace clippy, and the full offline workspace test suite. PR #314 is stacked on #313/#246 and can be retargeted to main after the zeta-projection PR merges.

### comment 5147972485 — 2026-07-31T22:24:23Z — auser

Update: #313/#246 has merged into main. PR #314 is now rebased/retargeted onto main and GitHub confirms a clean three-file diff (router provenance code, focused tests, and docs).

### comment 5154115941 — 2026-08-02T00:08:03Z — auser

Completed by merged PR #337 (merge commit `2c7712e`). The PR maps and satisfies all DoD items: uor-addr provenance fields at index time, corpus verification via `verify_corpus_provenance`, and retrieval exposure through `ResonanceResult.provenance`. Router gates passed. Closing as complete.


---
_Comment/review ids in this file: 5147966482, 5147972485, 5154115941_

