# #197 — feat(compiler/eval): Scaled D3 observation corpus compilation and M.V.G. evaluation certification

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-26T01:31:53Z
- **Updated:** 2026-07-26T04:03:35Z
- **Closed:** 2026-07-26T04:03:35Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/197
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: Master Roadmap Epic
Relates to: Phase 4 (Gate C / M.V.G. checkpoint) of `docs/r4_graph_compiler_implementation_plan.md`.
Source: `crates/uor-r4-graph-cli/src/lib.rs`; `crates/uor-r4-graph-certify`.

## Motivation

Initial smoke-test compiles used tiny parameters (`--target 20000` tokens, `--sequence-length 128`), producing a D3 held-out cross-entropy of `37.15` bits/token. To achieve the Minimum-Viable-Graph (M.V.G.) quality checkpoint, observation compilation must scale to `--target 500000` tokens and `--sequence-length 256`, followed by formal D3 certification via `r4 evaluate-report`.

## Scope

- **Scaled Observation Compile**: Execute compiler over 500,000 teacher observation tokens with sequence length 256:
  ```bash
  cargo run --release -- compile \
    --source .uor-models/sources/smollm2-135m-instruct \
    --output .uor-models/compiled/smollm2-135m-instruct-scaled \
    --target 500000 \
    --sequence-length 256
  ```
- **Evaluation Certification**: Execute `r4 evaluate-report` to generate content-addressed `instruction-eval.json` metrics over held-out D3 splits.
- **Metric Verification**: Validate that cross-entropy bits/token over teacher floor drops below `+15.0` bits/token and teacher-argmax agreement exceeds `35.0%`.

## Non-Goals

- Modifying the underlying certificate format or report CIDs.

## Affected Crates / Modules / Files

- `crates/uor-r4-graph-cli`
- `crates/uor-r4-graph-compiler`
- `crates/uor-r4-graph-certify`

## Deliverables

1. Scaled observation corpus compiled to `.uor-models/compiled/smollm2-135m-instruct-scaled`.
2. Content-addressed `instruction-eval.json` evaluation report certified by BLAKE3 CID.
3. Verification report documenting fidelity improvements over the smoke-test baseline.

## Acceptance Criteria

- `instruction-eval.json` generated cleanly with verified CIDs.
- Teacher-argmax agreement increases over baseline (`20.36%`).
- Four gates green (`cargo test`, `cargo clippy`, `cargo fmt`, `no_std`).

## Formal Claim Classification

- "Scaled observation compilation achieves lower D3 cross-entropy bits/token than smoke-test baseline": **Guarantee**, status **Witnessed** (Certified evaluation report).

## Test Requirements

- **Unit:** Scorer distribution tests in `uor-r4-graph-certify`.
- **Benchmark:** `evaluate-report` benchmark execution over held-out D3 split.


## Comments (1)

### comment 5081406002 — 2026-07-26T01:32:04Z — Casey-allard

Parent Master Epic: #198


---
_Comment/review ids in this file: 5081406002_

