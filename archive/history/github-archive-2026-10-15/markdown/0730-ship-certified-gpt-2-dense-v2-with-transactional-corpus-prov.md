# #730 — Ship certified GPT-2 dense v2 with transactional corpus provenance

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T02:45:49Z
- **Updated:** 2026-10-09T14:17:17Z
- **Closed:** 2026-08-16T08:07:37Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/730
- **Merged:** 2026-08-16T08:07:37Z
- **Merge commit:** 830aafde4c571e6356ee920ef4cc1108eb493997
- **Base / Head:** main / UOR-Foundation:issue-704-certified-dense-v2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This PR completes the uor-r4 work tracked by #704, #728, and #729.

- Implements certified native GPT-2 dense v2 for all 48 Conv1D projections plus the tied output head, with exact serial, trace, and genuine row-reuse batch semantics against the pinned exact reference.
- Carries one inseparable attention+dense execution identity through observation, recorded corpora, compiler/CLI/API/certifier paths, managed server publication, evaluation, cover, score, status, and serving.
- Makes recorded-corpus publication transactional: bounded no-follow readers, sorted common authority, crash recovery, binding-last commits, fixed inventories, managed multi-root CAS promotion, and BUSY-safe startup/reload.
- Replaces the historical scale-prefix writer with a bounded streaming fixed-partition Rust derivation, supports all 12/32/48/88-byte record eras plus opaque aligned hidden rows, and repairs the four-root scale workflow.
- Deprecates the Python writer and appends the #531 scaling-law honesty correction without running a new long measurement.

## Dense evidence

- Whole GPT-2 gate: 539 calls / 1,465,211 lanes.
- Paired median ratio: 0.694987183x.
- One-sided upper bound: 0.703876225x.
- Zero accepted whole-model fallback and zero hot-path allocation in the pinned gate.
- Immutable dense v1/v2 registry records and joint attention+dense era validation.

## Verification

- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --all -- --check`
- graph-format no-default and alloc ladders
- claim-wording gate and R5 audit
- graph CLI 125/125, graph compiler 199/199, server 174/174, scale workflow 6/6
- Kani, PR fast gate, register conformance, all-features, cargo audit, wasm, Gate C stub, and fuzz stub on head `6f1f53ff`
- independent final reviews: P0/P1/P2 = 0

## Repository boundary

All implementation is in `uor-r4`. No Casey-authored `uor-matmul` PR exists or is required. `uor-matmul` remains the pinned exact fallback/reference only.

Closes #704.
Closes #728.
Closes #729.

## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 5305398275 — 2026-08-16T02:48:33Z — Casey-allard

## Clean handoff checkpoint — 2026-08-15

The branch is pushed and clean at `a902dbc2f7429727811bc5198387fa2a1a74f146` (`WIP(#728): checkpoint corpus transaction and server CAS handoff`). This PR must remain draft; the exact Phase-C review still requests changes.

### Durable state

- #704 model-source dense lane is independently CLEAN: all 48 GPT-2 Conv1D projections plus tied head, exact serial/trace/true-batch parity, zero hot allocations, zero fallback in the accepted whole-model census.
- Accepted whole-model evidence: 539 calls / 1,465,211 lanes; paired median 0.694987183x; one-sided upper 0.703876225x.
- #728 Phase B lower/compiler/CLI transaction lane was independently CLEAN before Phase C.
- Current committed Phase-C file pins:
  - `src/server.rs` `2f8af67caa267341781af6fb0fe64b6e0bb0148a50dc0dd53b6be473357a7071`
  - `recorded_corpus.rs` `e7a159fc44bb9f250746839bed1d79cb3b90e7375dd009fc6c967a1dc08346a8`
  - graph CLI `lib.rs` `0c975b2590d914da1e28e623414327af2126af02d2a8896baa834f968593a749`
  - observation `61bdeada5c4b003a2192faabd59c5b8b6889a22b73fded567f961eb6a7dfe4e8`
  - graph compiler `lib.rs` `3fcd975d2e82323a5d93f4faa9e7a5bf1f022d45f9d0861fb2dea0786d60e64d`
- Evidence at this checkpoint: lower transaction 59/59; server module 158/158 before the final cap-only patches; final sparse cap regressions pass; root/compiler/CLI all-target clippy with `-D warnings`, fmt, and diff-check passed. No ignored clocks or full workspace gate ran.

### Remaining #728 blockers

1. **P1 prefix preparation/recovery:** accept only exact requested P/K identity prefixes for pre-attention or pre-dense, optionally with the canonical attempt marker, before full transaction recovery. Refuse mismatched P/K, attempt-only, binding-present, or payload roots; exclude the exact attempt marker from prefix copying.
2. **P1 mixed private-stage adoption:** recover lower binding `.tmp`/`.writing` first, then validate the entire retained copied prefix against the persisted/current typed base before every binding-present reuse. Pin A-to-B during copy, refusal, A restoration, and safe retry.
3. **P1 direct cover/score writers:** direct graph CLI `cover` and `score` currently rewrite completion-bound members without the common bundle guard. They must refuse an ancestor completed-bundle root or participate in the same exclusive transaction and republish completion atomically. Add startup/reload BUSY plus no-mutation tests.
4. **P2 completion wire pin:** add literal `uor-r4-compiled-bundle-completion/1` canonical JSON, trailing newline, and literal BLAKE3 CID.
5. **P2 legacy report cap:** classify legacy `compile_report.json` under the 1 MiB report cap while leaving `compiled.r4g1` streamed; add sparse preservation coverage.

### #729 remains pending

The scale sampler still needs bounded streaming instead of source+output materialization, the fixed-partition 12/32/48/88 fixture, four-root script layout, exact-full-N lane, actual-N reporting, crash/BUSY/residue coverage, and Python/docs deprecation/honesty correction. See #729 for the existing blocker audit.

### Resume order

1. Repair the three #728 P1 classes together.
2. Add the two #728 P2 tests/caps.
3. Run focused new tests, then full `server::tests`, lower transaction tests, root/compiler/CLI clippy, fmt, diff, and wasm check.
4. Re-pin exact hashes and request independent P0-P2 review.
5. Finish #729, run final workspace gates, replace WIP commit wording if desired, and mark this PR ready only after all issues have evidence.

### Environment handoff

- Worktree: `/Users/casey.allard/uor-r4/.worktrees/i704dense`
- Branch: `issue-704-certified-dense-v2`
- Remote head: `a902dbc2f7429727811bc5198387fa2a1a74f146`
- `git status` is clean.
- No Cargo process or agent is running.
- Free disk: about 54 GiB. Cleanup removed only reproducible Cargo `target/` and incremental artifacts; no source, model/corpus fixture, evidence, or worktree was deleted.
- All implementation stays in `uor-r4`; no upstream `uor-matmul` PR exists or is required.

### comment 5306127625 — 2026-08-16T06:30:48Z — Casey-allard

Pushed reviewed #728 checkpoint `9aeecc2c` (`fix(#728): make compiled corpus publication transactional`).

Evidence at this commit:
- independent exact static review: P0 0 / P1 0 / P2 0 for accepted #728 scope
- `cargo test --workspace --offline`: PASS
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: PASS
- `cargo fmt --check`: PASS
- graph-format no_std and no_std+alloc checks: PASS
- claim-wording gate: PASS
- `xtask audit-limits`: PASS (R5 clean)
- graph CLI 119/119 across ten consecutive runs while closing the hard-link alias cleanup race
- graph compiler 197/197; server 174/174

This commit records the completed corpus/server transaction work on GitHub. The PR remains draft because #729 sampler/scale workflow work is still outstanding; no merge readiness is claimed yet. Raw public capability cleanup remains an explicitly deferred hardening item.

### comment 5306381490 — 2026-08-16T07:45:35Z — Casey-allard

## #729 checkpoint pushed

Commit `6f1f53ff` completes the bounded fixed-partition sampler and corrected scale workflow tracked in #729.

The implementation streams selected rows under one sorted source/destination transaction, preserves exact source partition metadata and execution provenance, supports 12/32/48/88-byte rows plus opaque aligned hidden widths, commits binding last, and adds exact-full/undershoot/crash/BUSY/alias/idempotence coverage. The script now uses sibling input/compiled/cover/score roots and reports verified actual/train/held counts; the retired Python writer and #531 interpretation are corrected without running a new measurement.

Pre-push gates: full workspace tests, workspace clippy -D warnings, fmt, both no_std ladders, claim wording, R5, Bash 3.2 syntax, graph CLI 125/125, compiler 199/199, workflow 6/6. Independent final review: P0/P1/P2 = 0.

### comment 5306397026 — 2026-08-16T07:50:00Z — Casey-allard

## Merge-queue handoff

PR #730 is ready, all pull-request checks are green, and head `6f1f53ffd126ce78f81f6b325be9bf518e790d0f` is enqueued at position 1.

Local handoff state:

- worktree clean;
- local and remote branch heads match;
- commits are `9aeecc2c` (#728 transaction/integration) and `6f1f53ff` (#729 streaming scale workflow);
- PR body now closes #704, #728, and #729 on merge;
- no uor-matmul branch or PR is involved;
- no long measurement is running.

Next action: monitor merge-queue checks. After merge, verify the merge commit on `main`, confirm all three issues auto-close, and add the merge-commit evidence to any issue that remains open.


---
_Comment/review ids in this file: 5305398275, 5306127625, 5306381490, 5306397026_

