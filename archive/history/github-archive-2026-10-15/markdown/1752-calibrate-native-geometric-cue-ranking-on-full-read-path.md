# #1752 — Calibrate native geometric cue ranking on full read path

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T19:48:59Z
- **Updated:** 2026-10-09T13:17:22Z
- **Closed:** 2026-10-07T00:07:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1752
- **Merged:** not merged
- **Merge commit:** c971bb02e4535db33a3eb1e4b1dbe3779f901bfe
- **Base / Head:** codex/geometric-cue-calibration-20261005 / UOR-Foundation:codex/geometric-cue-fit-20261005
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The legacy cue fitter starts from zero and omits calibrated endpoint actions, so it cannot measure whether cue learning improves the current reader. Add explicit cue-calibration modes that warm the existing native cue and optimize cue coefficients against the full Copy/Period/Stop token distribution, with all records admitted and labels used only after the native read.

Roots/Copy and prefix/endpoint coefficient bytes remain frozen. Each changed cue honestly recompiles the prefix/endpoint binding chain, saves derived metadata and independently reloads it. Zero-update admission checks complete canonical and own-prefix replay, full128 gradients, and actual B8 cost. The 64-update fit includes baseline zero in ordinary CE selection and loads diagnostic labels/predictions afterwards. Scorer outcomes distinguish exact source choice, first emitted token and complete replies, including pair outcomes and row gains/losses.

Independent exact-head source review passed at 86bbdaae3e487349b5dafbcade2bddfb20af445e. Scoped pod compilation/tests are running; no fit or improved attention claim yet. Tests cover stale data/donor/B8 admission, mode contracts, and sibling observation rejection. Review found and repaired incomplete fixture and loaded-mode enforcement; an adjacency concern was withdrawn after tracing the existing layout validator.

Stacked on #1751. References #1552 and #820. Parent delivery, executed final-head validation and protected merge remain outstanding.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6001861730 — 2026-10-05T19:52:07Z — Casey-allard

Exact86bbdaae scoped pod execution PASS:1296Rust/Cargo/.cargo sourcehashes; rustfmt --check; all28 geometric-bank-fit tests; observation overlay-rejection focusedtest; release geometric-bank-fit build. Allcommand exitcodes0,70.178s total, peak ownprocessgroup2.551GiB, Linuxx86_64 CPU16affinity/CUDAhidden. Exact-head independent source review PASS. Zero-update actualreader admission now started separately; no optimizer or learned result yet. Library#1751 remains independently checked at19bef. /root/codex2.6GiB; finished checks copied to durable /workspace alongside admission results.

### comment 6027852380 — 2026-10-07T00:07:24Z — Casey-allard

Superseded by #1810, #1812 and #1813, all now merged through the protected queue. The selected active native runtime/CUDA/learning dependency closure is on main. Historical experiment drivers and unused API variants remain on this pushed branch as retained evidence; no branch is deleted and no mechanism-family verdict follows. Exact-head checks and preservation receipts: #1813 and #820 (triage issuecomment-6026806242).


---
_Comment/review ids in this file: 6001861730, 6027852380_

