# #298 — feat(score): #243 Phase C part 2 — witness-attested graded codes (option A) + metric-consistent trend re-pin

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-30T05:45:04Z
- **Updated:** 2026-10-09T14:16:55Z
- **Closed:** 2026-07-30T06:13:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/298
- **Merged:** 2026-07-30T06:13:56Z
- **Merge commit:** 97495dbce835d12da0b7604455afb50dbe1c8031
- **Base / Head:** main / UOR-Foundation:issue-243-sig-path-adoption
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements the part-2 maintainer decision (Casey, 2026-07-30, on #243): the serving surface adopts the artifact's declared metric via a **4-byte attested code in the witness** rather than carrying the integer bundle.

- `ScoreWitness.input_code` — attested by the producer, replay verifies consistent consumption (coded replay + equality check); code derivation stays proven at certify time (kernel==plain witnesses).
- `score_candidates_coded` / `score_step_coded_with_recent` — coded entry points; sign-metric artifacts may omit the code (the sig IS their metric); TLA6 without a code is a contract error, never a silent sign fallback.
- `R4Engine::derive_sig_code` — serving windows attest metric-respecting codes through `predict_decision`.
- Gate C harness — one metric-consistent code per position (`code_plain`), consumed by baseline and attested to every coded row; both repetition probes likewise.

## The headline measurement

The pinned 1.11%/15.9742 "collapse" row was the writer/reader mismatch itself (sign-code probes on a dot-keyed store), not the system. Metric-consistent probes measure:

| | mismatch era (#296 pin) | **consistent (this PR)** |
|---|---|---|
| rule12 | 1.11% / 15.9742 | **31.63% / 9.1181** (best recorded fixture row; sign-era tie was 30.02/10.24) |
| status split | 291 / 29,725 / 20 | **25,853 / 4,182 / 1** |
| strict EXCT miss | 99.03% | **13.93%** — Gate C valid AND the graph path runs on 4,182 real probes |
| baseline (dot store) | 22.98 / 9.51 | 33.83 / 11.95 |

Trend re-pin travels in this PR; self-consistency mode validates it (first live use of the #297 two-mode check). The graph-runtime EXCT reader (chat/wasm) is token-keyed — outside option-A scope, recorded in the commit.

Remaining Phase C: C-row evidence pair on the TLA6-recompiled D3 bundle (in flight), BASELINE table, closes #230.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

