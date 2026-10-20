# #1702 — Isolate geometric context drift with frozen attention factor probes

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T23:04:53Z
- **Updated:** 2026-10-09T13:17:19Z
- **Closed:** 2026-10-04T23:06:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1702
- **Merged:** 2026-10-04T23:06:43Z
- **Merge commit:** b964e7a86e23edc78680c84716bb9f0b2a5e836d
- **Base / Head:** main / UOR-Foundation:codex/geometric-bank-factor-20261004
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The mixed-bank joint fit traded new bank probability gains for severe original-control regressions. This change adds a zero-update Rust factor probe that independently compiles parent0/full48 and crosses their context and geometric readout parameters, preserving all candidates, the actual context replay and one global Copy/Period/Stop alias reducer.

Actual full128 development evaluation identifies context drift as a material harm channel: learned context with parent readouts gives4/64 original completions; parent context with learned readouts preserves27/64 versus parent31/64 and lowers combined CE1.327649→1.305688. Full48 also produces2/64 complete bank replies, both dancer, with no complete query pair. Preserve these useful gains and every regression. The probe makes no optimizer updates, fresh predictions, reselection or crossed-model adoption. Next use ordinary bank readout learning with context frozen, retaining mutable geometry for subsequent directed relation/transport work.

Validation:9 focused Rust tests and optimized build pass on Linux CPU;14.241-second native probe enforces donor parameter/payload equality and complete retained canonical endpoint replay. Independent saved audit passes23,129 checks0errors; complete remote/local SHA256 file-set equality returns103files214,942,337B. Source/executable/cost and exact limitations are recorded. Formatting, diff and claim-wording checks pass. Same180min card oncecharged, no new rental/GPU/runner/localmodelCPU. Local independent BLAKE3/rawTensor/backward reconstruction and Metal remain unavailable or unperformed; no general-chat/attention qualification.

References #1552. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

