---
id: cpu-discrete-training-path
title: Port exact CPU reads, counterfactual updates and BVG-K
milestone: research
area: training
status: done
open_to: lab
needs_gpu: false
depends_on: [cpu-discrete-writer-core]
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [crates/uor-r4-cpu-train/**, Cargo.toml, Cargo.lock, docs/loop/tasks/cpu-discrete-training-path/**, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-discrete-training-path.md]
---

## Why it matters

The discrete learner is the strongest comparison-heavy CPU candidate, but its early failures came from read/proposal/admission interactions that a writer-only oracle cannot expose.

## Progress

Route, Flock and Relation reads, exact 16-lane proposal scoring, the chooser, single-coordinate apply, all-or-nothing BVG-K joint apply and deterministic checkpoint/restart now have independent finite parity slices. The batch boundary validates every committed lane before staging, all listed CUDA refusal statuses have immutability controls, and a restored child process reproduces the uninterrupted next decision, update, reading and checkpoint bytes. The CPU checkpoint writes the reference cursor fields while labeling its CPU development substrate and GPU qualification requirement.

## Done when

- Each read has finite parity tests against its source specification before the next read is added.
- Proposals and apply reproduce exact decision traces on deterministic fixtures, including rejected equal-loss moves.
- Admission-preserving acceptance and deterministic resume are tested across a restart.
- The CPU trainer writes the same checkpoint identity fields as the reference format and labels the compute substrate.
- One notebook row and a REPORT place every negative beside its configuration.

No corpus training, language capability or qualification claim is included in this item.

## Cancel if

The writer oracle fails to reproduce the stored observation, or the required source semantics cannot be implemented without editing Claude's active serving files. Fix the oracle first or wait for that claim to merge; do not widen scope silently.

## How to start

Read `crates/uor-r4-discrete/src/discrete.cu` for the executed semantics and the host source files `rows_source.rs`, `rowsk_source.rs`, `bvgk_source.rs`, `training.rs` and `objective.rs` for ordering and validation. Port reads one at a time and keep every later stage disabled until its finite parity test passes.

## Rules that apply

- Admission preservation is an acceptance condition.
- The loss, proposal and apply path is exact geometry, with no normalized exponential or dense product.
- A negative closes the tested update configuration, not the discrete mechanism.

## Sources

- `crates/uor-r4-discrete/src/discrete.cu`
- `crates/uor-r4-discrete/src/rows_source.rs`, `rowsk_source.rs`, `bvgk_source.rs`
- `crates/uor-r4-discrete/src/training.rs` and `objective.rs`
