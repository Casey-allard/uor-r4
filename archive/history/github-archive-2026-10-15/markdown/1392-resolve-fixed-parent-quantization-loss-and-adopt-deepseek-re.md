# #1392 — Resolve fixed-parent quantization loss and adopt DeepSeek review cadence

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-25T15:11:20Z
- **Updated:** 2026-10-09T13:17:48Z
- **Closed:** 2026-09-25T15:12:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1392
- **Merged:** 2026-09-25T15:12:38Z
- **Merge commit:** c465c70915c35947d0277e993cb05fa6c9e1d9ac
- **Base / Head:** main / UOR-Foundation:codex/precision-factorial-20260925
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Decision

Complete the fixed-parent parameter/interface precision diagnosis. Parameter quantization dominates immediate mean likelihood loss in both retained projected learners. No model is promoted; both earlier quantized recipes retain their original rejections. Recommend one paired learning of neighboring integer-code choices at fixed parents, scales, bit widths and interfaces, using full-context language loss. That candidate is NOT_RUN and retains the original five acceptance gates.

## Changes

- Add guarded evaluation-only `PrecisionMode::{FF,QF,FQ,QQ}` and `joint-evaluate-precision` in Rust. Keep legacy loading/serving unchanged; reject training, serialization/export and incompatible sessions on diagnostic views.
- Execute all four combinations in both step-8,348 parents at B16/T256, on the same 249,856 targets and actual generated/source-edited response panels. Preserve every prior artifact.
- Verify fresh FF/QQ endpoints against all retained target bytes and complete response structures, excluding only elapsed time. Publish complete arithmetic, 40 free generations, 256 source responses, exact identities and the reproducible non-model analysis script.
- Synchronize current state, roadmap, evidence/navigation and training CLI documentation.
- Apply the owner's September 25 cost correction: DeepSeek is the default specialist route, including architecture. Kimi requires new explicit owner direction. Reuse evidence/reviews and budget/report total coordination and delivery time.

## Result

Comparison-tail NLL (quaternion / ordinary): FF 2.113220567 / 2.098025647; QF 2.150291052 / 2.147912004; FQ 2.113217828 / 2.098039459; QQ 2.150319798 / 2.147928712. Parameter cost with floating interfaces is +0.037070 / +0.049886 nats. Even QF misses the original +0.05 gate against continuous. Small interface mean effects do change actual text. The result does not identify a tensor culprit or guarantee four-bit recovery.

## Executed validation

- 25 focused optimized Rust tests passed, including all three new precision tests, packed reload, gradients, causal sessions and codec/projection invariants; test execution 2.16 seconds.
- Optimized training CLI built; executed source `49dfd0b15486ba3a7a763c598257f651e857279f`, executable SHA256 `cf5ce19a5eb63b674bf9ee23fe671ceed477cb26ad8d5e3e796c974c04a01f9f`.
- Eight actual evaluations complete; four retained endpoints reproduce exactly at the declared scope; 16 report roots verified and eight bound input file sets unchanged.
- `cargo fmt --check`, `python3 scripts/check_claim_wording.py`, the full base-to-head `git diff --check` for authored files, source/dependency identity and published-evidence byte comparisons passed after documentation changes. No model rerun was needed. The full staged whitespace check flags 18 trailing-space lines in literal generated text; that exact-copy output file is preserved and excluded from the authored-file whitespace check.
- The five historical queue status names are compatibility acknowledgements, not tests.

## Resources and limits

Model pairs ran concurrently through RDC on the same M1: 183.47 seconds total pair wall time, peak sampled aggregate model RSS 1.54 GB. Compilation was approximately seven minutes. The full preparation/analysis/review/documentation cycle is charged to the shared ledger; its overhead is acknowledged in the review and addressed in the workflow. Prospective complete-cycle limit is two hours with a 2 GiB new-allocation ceiling and a physical reserve; no new fitting, paid external training compute or cleanup occurred. Delivery-tail accounting follows verified merge.

All paths remain F32 emulators. Integer serving, bounded trained admission, D5 sparsity, Hamiltonian attention, geometric advantage, useful general language and physical energy remain unqualified. Final holdout was not opened.

References #973
References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

