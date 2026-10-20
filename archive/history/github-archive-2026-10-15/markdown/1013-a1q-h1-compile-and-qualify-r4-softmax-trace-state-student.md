# #1013 — A1Q-H1: compile and qualify R4 softmax trace state student

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-30T15:44:08Z
- **Updated:** 2026-10-09T14:16:49Z
- **Closed:** 2026-08-30T15:46:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1013
- **Merged:** 2026-08-30T15:46:21Z
- **Merge commit:** f193939bbb40d0fa8c6a3d4a5015ea6439ad7bf5
- **Base / Head:** main / UOR-Foundation:issue-1011-r4-softmax-trace-state-student
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements and completes R4SoftmaxTraceStateStudentV1 from the frozen #1010 construction trace. The three-phase causal run passed execution-integrity gates and failed its predeclared promotion gate cleanly.

## What changed

- canonical fail-closed teacher-trace document and bundle reload with CID, shape, order, truncation, trailing-byte, and tamper checks;
- 278-byte four-bank R4/Spin recurrent state and matched 120-value geometric/plain arms;
- deterministic transport-permuted control and unchanged suffix baseline;
- separate compile, source-free seal, and reveal commands with strict input schemas and work ledgers;
- deterministic artifact/state serialization, replay, causal-input provenance, and cycle checks;
- exact-executor content-contract binding for the new production source;
- synchronized README, research, lifecycle, programme, evaluation, ADR, glossary, formal vocabulary, roadmap, and issue record.

## Authoritative result

All arms covered the same nine positions and 422,875 Q16 teacher mass:

| Arm | Covered CE | Teacher top-1 | Actual-next top-1 |
|---|---:|---:|---:|
| suffix | 2.660721032 | 3/9 | 2/9 |
| plain recurrent | 2.660770919 | 3/9 | 2/9 |
| geometric recurrent | 2.660705367 | 3/9 | 2/9 |
| transport-permuted | 2.660729215 | 3/9 | 2/9 |

The geometric improvement over suffix was 0.000015665 nats. Its separation from the permuted control was 0.000023848 nats versus the frozen 0.10 requirement. No top-1 decision changed; every arm retained the same period-two Scotland loop.

Exact replay passed and every forbidden runtime-read counter was zero. Terminal:

    STOP_R4_SOFTMAX_TRACE_STATE_STUDENT_REPAIR_OR_RETIRE_REPRESENTATION

This is a negative for the frozen 4D signed-reduction/token-derived state cell, not for ordinary R4/Spin causal softmax attention.

## Evidence identities

- state artifact: blake3:b617fc38e7bef1cdea76991f6e5e7cc653118451d63bcbd595f8ffd7e247ae7b
- construction freeze: blake3:67cf67bb46b94cf5644b8dde286e89adb7e49159b3749790dffb500d8047fedb
- source-free seal: blake3:64587526f7883ab046e884a28b6af7e9e89818c9ead2039f8c995de7fb483060
- structured result: blake3:dc04a8a8b21750799db2d451c8237d1e62cf90ffa74561fb54272b1e9704c824

## Local verification

- cargo test --workspace --offline --no-fail-fast
- cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
- cargo fmt --check
- both graph-format no-std ladder checks
- wasm32 library check
- claim-wording gate
- focused loader, state core, orchestration, strict CLI, replay, and exact-executor contract tests

The protected checks are delivery acknowledgements; the local commands and frozen artifacts above are the validation evidence.

## Next

#1012 is open, assigned, and wired as the next child/blocker of #973. Its construction-only four-fold observability ladder localizes the first signal-loss boundary before any scale, state widening, resonance, exact lowering, WASM, or release work.

Closes #1011
References #973
References #1012

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

