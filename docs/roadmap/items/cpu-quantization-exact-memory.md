---
id: cpu-quantization-exact-memory
title: Add CPU quantization-aware training and exact memory
milestone: research
area: training
status: done
open_to: lab
needs_gpu: false
depends_on: [cpu-shared-vector-train, cpu-discrete-training-path]
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [crates/uor-r4-cpu-train/**, Cargo.toml, Cargo.lock, docs/loop/tasks/cpu-quantization-exact-memory/**, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-quantization-exact-memory.md]
---

## Why it matters

Earlier learned-rounding work retained likelihood but bounded admission lost source rows. Current exact export and turn-memory mechanisms now provide a direct way to measure whether a training gain survives serving.

## Progress

Hamming reads and exact turn memory have finite parity with serving. The supported CPU FBS session also has a versioned, artifact-bound checkpoint covering recurrence/history, Hamming payload caches, opening/reply anchors and reply position; a child-process restore and reset control reproduce the uninterrupted next step exactly.

Deepseek-2 adds a finite no-Hamming/no-memory quantization-shadow checkpoint: accepted Q12 table deltas are applied to the continuous source shadow, every update must re-export the exact artifact, and the gate distinguishes exact gain, admission loss, export loss and float-only `EXPORT_LIMITED` gain.

## What is missing

This component item is delivered. A trained FBS float snapshot and matched query panel are still unavailable, so trained-artifact, complete-reply and fresh-phrasing outcomes remain `NOT_RUN` under the downstream pilot rather than being silently treated as passes.

## Done when

- Hamming and exact-memory operations match the current serving reference on finite fixtures.
- Every checkpoint records its exact export gap on the same evaluator and panel.
- An admission-preserving update rule is tested against a matched control.
- Process restart and memory reset produce the registered deterministic token continuation.
- One notebook row and REPORT classify fit, admission and complete-reply outcomes separately.

## Cancel if

A current serving claim changes the format under the unit. Merge it forward and re-check parity before training; never infer export equivalence from float behavior.

## How to start

Read the current Hamming read, turn-memory implementation and exporter parity records. Build exact CPU fixtures for one read and one memory restore before connecting either to training. Measure the export gap on a checkpoint that already exists.

## Rules that apply

- The exported integer artifact is the comparator, not a rounded approximation.
- Memory identity stays separate from geometric address.
- A lower fit score without source retention or complete replies is not a gain.

## Sources

- `crates/uor-r4-integer/src/stack/hamming.rs` and `memory.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-training/src/native_vector_writer.rs`
