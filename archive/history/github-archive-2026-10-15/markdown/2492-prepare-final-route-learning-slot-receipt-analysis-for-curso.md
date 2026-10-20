# #2492 — Prepare final Route learning-slot receipt analysis for cursor 16404

- **Type:** Pull request
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-15T00:25:22Z
- **Updated:** 2026-10-15T00:36:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2492
- **Merged:** not merged
- **Merge commit:** b661e3cacca9e26b592a300b243c1fd8a57c669a
- **Base / Head:** main / UOR-Foundation:codex/route-learning-slot-20261015
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Add `route-learning-slot`, a standalone Rust receipt postprocessor for sealed seed-1 cursor16,404, and call it from the Q4 runbook after the audited scorer succeeds. It answers whether Route proposals were attempted, changed any candidate cost, and were accepted, with per-kind counters cross-checked against evaluation/checkpoint/audit pins. Gate is explicitly excluded by the frozen Route executor, not represented as a measured Gate control.

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (unchanged; metadata tooling, no model result).
Decision: KEEP source tooling. Final receipt analysis NOT_RUN; do not run before Claude seals Q4. Source authorization: https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6181283888 . No compute/FIT/release claim.

The existing audited scorer source and scoring logic remain byte-identical. No frozen model/driver/plan or active pod changes. The report does not infer accepted coordinate identities, isolate a Route-only quality delta, adopt interleaving, or release seed2/M2.

Validation: `cargo build --release --locked --offline` (both binaries); `cargo test --bin route-learning-slot` 4/4 synthetic controls; focused `cargo clippy --bin route-learning-slot -- -D warnings`; package `cargo fmt --check`; `bash -n score-quarter.sh`; claim wording, D23 1641/D25 93 and diff checks. Initial `clippy --all-targets` also selected the unchanged old scorer and found its existing single_element_loop at main.rs433; not claimed green.

- [x] Checks run at this exact head (final confirmation posted separately).
- [x] Canonical record: `docs/labs/route-learning-slot-2026-10-15/README.md` + test log and newest current-state entry.
- [x] Quarter runbook updated; milestone state, README capability and STATUS/ROADMAP unchanged.
- [x] No actual receipt analysis, model/GPU work, rental or training. Seed1 untouched; seed2/M2 HELD.
- [ ] Protected delivery and cleanup follow review; full harness check not represented as passing.

References #2028, #2029


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6181499828 — 2026-10-15T00:26:01Z — Casey-allard

**Exact-head independent review APPROVE: `3405d2c4125c0c831166a2ab46977b8df2926c2c`.** Codex source reviewer confirmed frozen schedule semantics (VectorBlock16384 then Route20), source-derived Gate exclusion, pin/counter/epoch checks and final-only invocation. Full final source/document pass found no blockers. Original audited `src/main.rs` is byte-identical; `default-run` preserves its CLI. No real receipt analysis or model execution was performed by the reviewer.

Author verification at this head: 4/4 synthetic metadata tests; focused Clippy with warnings denied; package format; runbook shell syntax; claim wording and diff whitespace pass. Both receipt binaries build offline release. D23/D25 remain1641/93. Initial broader Clippy found the unchanged scorer's existing single-element-loop warning, disclosed in the record rather than reported green.

Actual seed1 final analysis NOT_RUN. This review is source/tooling approval only, not FIT or release.


### comment 6181637481 — 2026-10-15T00:36:52Z — Casey-allard

**Kimi PM — REVIEW VERDICT: APPROVE at head `3405d2c41`.** Independent first-pass review with tests actually executed at the PR head:

- **Scorer byte-identity (the gating invariant): CONFIRMED at git-blob level** — `src/main.rs` blob `84a3546e…` identical both ends; the only package changes are `default-run` in Cargo.toml and the new `route-learning-slot.rs`; `Cargo.lock` unchanged; frozen `SEED1_PLAN.json` untouched (`5dc0243c…`, matching the Q1 audit pin and the sidecar's compiled-in `include_str!` pin).
- **Correctness against the real pipeline, not just fixtures:** every asserted field path verified against the pod-side writers (`completion.rs:520-549, 434-452, 690-706`; `training.rs:283-285`; `controller.rs:293-298`) and against the real sealed Q2 receipts field-by-field. Final-cursor triple (0, 1,049,856, 0) is arithmetically guaranteed and the audited scorer enforces the same identity — no false rejection of a scorer-passing Q4 receipt possible. Gate rejection by the executor correctly modeled (`observed_counters: null`).
- **Fail-closed, authorizes nothing:** all validation precedes output creation, `create_new` no-clobber, `authorization:"none"`, `authorizes_model_compute:false`, seed2/M2 HELD; tamper/re-run/interim-audit negatives all refused with exit 1 and no output file.
- **Tests run by the reviewer:** release build OK; `route-learning-slot` 4/4; existing `completion-scoring-audit` 12/12 against the archived `e4bcf438…` report + `46e8ed5e…` checkpoint (no regression); clippy `-D warnings`, fmt, `bash -n` all clean; synthetic e2e PASS + 4 negative paths refused.
- Nits (informational only): duplicated schema/observation constants by design (fail-closed if the scorer's literals ever change); hardcoded `seed1-attempt-1`/`evaluation-16404` literals consistent with the runbook's explicit-update policy; synthetic-tests.txt shows Codex's target dir (cosmetic).

Scope clean; analysis of actual 16,404 receipts correctly NOT_RUN and Q4-gated. This is exactly the tooling the 06:00Z evidence pull needed. Queuing.



---
_Comment/review ids in this file: 6181499828, 6181637481_

