---
id: native-space-state-contract
title: Define one persistent native training/serving state and its operator parity
milestone: research
area: architecture
status: ready
open_to: lab
needs_gpu: false
depends_on: []
scope: [AGENTS.md, CHARTER.md, PLAN.md, Cargo.lock, Cargo.toml, docs/harness/native-space.json, docs/loop/tasks/native-space-state-contract/**, docs/model/**, docs/mechanisms/**, docs/research/notebook/float-training.md, docs/roadmap/items/native-space-state-contract.md, crates/uor-r4-core/src/native_geometric/**, crates/uor-r4-discrete/**, crates/uor-r4-native-vector/**, crates/uor-r4-integer/Cargo.toml, crates/uor-r4-integer/src/stack/**, crates/uor-r4-integer/src/fbs_snapshot.rs, crates/uor-r4-integer/src/discrete_d11.rs, crates/uor-r4-float-train/src/**, crates/uor-r4-harness/src/checkdispatch.rs, crates/uor-r4-harness/src/harnessrules.rs, crates/uor-r4-harness/src/ruleinventory.rs, crates/uor-r4-harness/rules/**, crates/uor-r4-harness/tests/**]
---

## Why it matters

<!-- native-space-contract: Native default: one persistent native geometric state uses one declared integer or fixed-point representation for token addresses, vector coordinates, carry, routes, memory keys and reads, learned state, gradients and optimizer updates; training and serving use the same typed operators; route choices, carry state, read choices, memory addresses and token ids are bit-identical on the same prompt and starting state; the artifact losslessly serializes that state instead of an F32 master model or an export gap; floating point remains reference or diagnostic code only. -->

D85 makes one persistent native geometric state the default model path. The current float
trainer and D11 exporter demonstrate a historical route: F32 master state is rounded,
exported and measured against the served artifact. It cannot show that training and serving
execute one transition.

Before another model is trained on the default path, the project needs one executable state
contract shared by both phases.

## What is missing

**November 17, 2026.** The Q31 CUDA updates, integer recurrence and bounded fixtures are on main. Acceptance still calls for the complete operator inventory, raw-word replay over every required field and a recorded planted fault. The written-state S0–S6 contract identifies the fields and operators for the current model. [PR #3962](https://github.com/UOR-Foundation/uor-r4/pull/3962) · [PR #4010](https://github.com/UOR-Foundation/uor-r4/pull/4010) · [PR #4046](https://github.com/UOR-Foundation/uor-r4/pull/4046)

There is no single type that declares which token addresses, vector coordinates, SpiralCore
carry, routes, memory keys, memory reads, learned state and optimizer state cross every
training and serving boundary. There is also no source-level operator registry proving that
the CUDA training kernel and the integer serving kernel implement the same transformation.

## Done when

1. One versioned native-state schema names every carried field, numeric domain, width, scale
   and lifetime.
2. One source-level operator registry maps every native transition to its CUDA training,
   integer serving, CPU oracle and WASM implementations where present.
3. A finite replay fixture starts from the same prompt/state in training mode and serving
   mode and compares raw words bit-identically for routes, carry, read choices, memory
   addresses and emitted token ids.
4. The fixture fails when one backend changes an operator, a scale, a route or a carry field;
   the planted fault is recorded.
5. The document names which current float/exporter paths are legacy or diagnostic code and
   which files implement the new state.
6. `cargo test`, Clippy on touched Rust crates, `uor-harness check`, and the item's focused
   replay checks pass.

No training run, speed claim or language result is part of this item.

## Cancel if

The source inventory cannot identify one native transition that exists in both training and
serving without inventing a new mechanism. Record the gap and stop before implementation.

## How to start

Read D85, CHARTER §3.7, `.agents/skills/native-space-cycle/SKILL.md`,
`docs/harness/native-space.json`, the pipeline map and the current integer session /
`crates/uor-r4-float-train` call chains. Produce the state and operator inventory before
editing kernels.

## Rules that apply

- One state and one typed operator definition (D85).
- Integer-only serving (D11), with no float or transcendental instruction.
- No matmul/GEMM and no softmax/squashing (D23/D25).
- Rust first (D27); CUDA kernels are ahead-of-time modules (D29).
- Tests compare raw integer words or bits; rounded values are not parity.

## Sources

- [`CHARTER.md`](../../../CHARTER.md) §3.7
- [`DECISIONS.md`](../../integration/DECISIONS.md) D85
- [`native-space.json`](../../harness/native-space.json)
- [`.agents/skills/native-space-cycle/SKILL.md`](../../../.agents/skills/native-space-cycle/SKILL.md)
