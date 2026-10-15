# #329 — feat(transformerless): #318 Phase B — integer residual wiring, era TLA7

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T18:05:24Z
- **Updated:** 2026-10-09T14:17:12Z
- **Closed:** 2026-08-01T18:52:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/329
- **Merged:** 2026-08-01T18:52:55Z
- **Merge commit:** cff684fefc672860c3ca0741125631aad9b7d430
- **Base / Head:** main / UOR-Foundation:issue-318-phase-b
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #318. Implements `docs/dot_residual_phase_b_design.md` Phase B (merged via #323, Phase A.5 via #325; reproduction + A.5 evidence in the #318 comments).

**Draft pending the TLA7 quality-confirmation run** (kernel residual path vs the certifier's A.5 row numbers on the 500k corpus — results posted here and on #318 before this is marked ready).

## What lands

- **Compiler**: `Compiled` gains `resid_cb` (per-stage i8 centroid copies, IEEE-exponent po2 stage scale, libm-free), `resid_scale_shifts`, `norm_fold_const` (stride-8 train median of log2(L1/L2), D2-canonical float functions). Serialized under new artifact era **TLA7** (TLA6 payload + CONST + shifts + copies ≈ +288 KB), default-on with `R4_TLESS_TLA7=0` opt-out, mirroring the TLA6 mechanism.
- **Runtime**: TLA7 artifacts route to `assign_code_for_bundle_resid` — po2 norm fold (L1 bit-length − CONST, one arithmetic shift per dim; division-free), then per-stage dot argmax + winning-centroid integer-copy subtraction. Add/sub/shift/compare/table-read only; P-4 scan clean. Allocation-free serving path preserved (new ZERO-allocation census phase). Pre-TLA7 artifacts unchanged; membership beam stays on the #281 rule pending Phase C's store-shape decision.
- **Witnesses**: kernel==plain (s>0, s<0, L1=0, negative-heavy), TLA7 roundtrip + truncation rejection + TLA6-unchanged, TLA5 fixture byte-identical.

## Design note for reviewers

The doc's literal fold spec (normalize to unit scale) is degenerate in integer arithmetic (truncates to zero); the implementation folds to unit·2²⁰ (`RESID_WORK_FRACTION`), which keeps the copies and folded work on a common scale and leaves argmax scale-invariant. Documented on the field; flagged in the doc/code gap note on #318.

## Gates (worktree, offline)

`cargo test -p uor-r4-core` (22 binaries, incl. allocation census + P-4) ✓ · workspace clippy `-D warnings` ✓ · fmt ✓ · `cargo check --workspace` ✓

## Explicitly deferred (Phase C, per the design doc)

κ re-pin + fixtures (tracks #327), certifier-side TLA7 witness rows, store-shape/beam decision on the #244 harness, BASELINE.md.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

