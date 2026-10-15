# #196 — feat(context): Multi-timescale context abstraction layer (Phase 8) & macro-region VSA sequence state

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-26T01:31:49Z
- **Updated:** 2026-07-26T02:19:03Z
- **Closed:** 2026-07-26T02:19:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/196
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: Master Roadmap Epic
Relates to: Phase 8 of `docs/r4_graph_compiler_implementation_plan.md` (PDF §8, §16-§17).
Source: `crates/uor-r4-core/src/transformerless/runtime.rs`; `crates/uor-r4-graph-runtime`; `vsa_algebra.rs`.

## Motivation

The baseline engine is constrained by an 8-token context ceiling (`WINDOW = 8`), resulting in "Markov Amnesia" for prompts requiring sequence history past 8 tokens. 

Plan Phase 8 specifies a **Multi-Timescale Context Abstraction Layer**, where micro-level 8-token sliding windows are aggregated into macro-level semantic regions (32, 128, 512 tokens) using Bott-Fock folding and holographic VSA algebra (`vsa_algebra.rs`), without allocating heap memory on the prediction path.

## Scope

- **Bott-Fock Sequence Aggregator**: Wire `crates/uor-r4-core/src/transformerless/bott_fock.rs` to maintain macro-region sequence states.
- **Hierarchical Graph Traversal**: Extend `crates/uor-r4-graph-runtime` to query multi-resolution region nodes (Level 0: 8 tokens, Level 1: 32 tokens, Level 2: 128 tokens).
- **VSA Vector Binding**: Use permutation and binding operations from `vsa_algebra.rs` to encode temporal sequence context into fixed-width state slots.
- **Zero-Allocation Property**: Maintain the zero-allocation steady-state guarantee during multi-scale context updates.

## Non-Goals

- Adding matrix multiplication or floating point ops to runtime inference (kernel remains integer-only per P-4).

## Affected Crates / Modules / Files

- `crates/uor-r4-core/src/transformerless/runtime.rs`
- `crates/uor-r4-core/src/transformerless/bott_fock.rs`
- `crates/uor-r4-graph-runtime`
- `crates/uor-r4-core/tests/vsa_algebra.rs`

## Deliverables

1. Multi-timescale context aggregation in `RuntimeState` across micro (8) and macro (32/128) token windows.
2. Hierarchical region candidate scoring in `uor-r4-graph-runtime`.
3. Allocation census test asserting zero allocations during multi-scale context updates.

## Acceptance Criteria

- Context history beyond 8 tokens influences prediction candidate scoring without degrading prediction speed.
- `allocation_census` test passes with 0 allocations on steady-state multi-scale generation.
- Four gates green (`cargo test`, `cargo clippy`, `cargo fmt`, `no_std`).

## Formal Claim Classification

- "Multi-scale context graph aggregation operates without heap allocations in steady state": **Guarantee**, status **Witnessed** (allocation census test).

## Test Requirements

- **Unit:** Bott-Fock state transition and VSA binding unit tests.
- **Property:** Context-length invariant tests (long sequence history preservation).
- **BDD / Cucumber:** Feature scenario testing sequence continuation across 128-token context windows.


## Comments (2)

### comment 5081405960 — 2026-07-26T01:32:03Z — Casey-allard

Parent Master Epic: #198

### comment 5081592979 — 2026-07-26T02:19:02Z — Casey-allard

Closing based on Maura's diagnostic: Chain 2 context encoding is mathematically restricted to an 8-token rolling window. Expanding it violates the R4G1 spec and corrupts the index. Replaced by Master Tracker Epic #201.


---
_Comment/review ids in this file: 5081405960, 5081592979_

