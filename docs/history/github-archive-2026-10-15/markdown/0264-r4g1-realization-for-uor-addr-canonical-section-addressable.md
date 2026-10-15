# #264 — R4G1 realization for uor-addr: canonical, section-addressable artifact addressing (+ vendored-copy sync)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T15:05:54Z
- **Updated:** 2026-08-01T00:46:33Z
- **Closed:** 2026-08-01T00:46:33Z
- **Labels:** enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/264
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
uor-addr ships canonical realizations for JSON, CBOR, ASN.1, GGUF, ONNX — but R4G1 artifacts are addressed by raw file-byte hashes (`file_cid`). A structural **R4G1 realization** (Merkle skeleton over the container's sections, in the style of the GGUF realization) yields addresses that are encoding-independent and **section-addressable**: EMIT / ROUT / store sections individually addressed under one artifact κ.

### Why
- Section addressing is the substrate immutable graph patches (the #30 RFC) verify against, and what partial region fetch (the region-object issue) composes with.
- Also the natural first upstream contribution from r4 to the standards program — a real-world realization exercising the uor-addr conformance discipline (ANALYSIS.md-style CPs, positive/negative vectors), which the W2 suites can consume.
- Includes the hygiene item: sync the ten stale ONNX-realization files in the vendored `uor_standards/` reference copy (the build deps are already at latest HEAD — verified 2026-07-28; only the gitignored reference copy lags).

### DoD
An `r4g1` realization (in-repo first; upstreaming decision recorded) with: canonical section skeleton, artifact κ stable under byte-irrelevant reserialization, per-section κs, round-trip conformance tests with positive and negative vectors; `certify`/serving artifact CIDs switched from raw file hashes to the realization; vendored reference copy synced.

*Companion to the region-object issue; feeds #259's provenance chains.*


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

