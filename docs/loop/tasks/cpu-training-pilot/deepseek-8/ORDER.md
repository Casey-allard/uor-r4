# ORDER: full-path FBS audit coverage, deepseek-8

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `942d0086a`.

## Goal

Exercise the served-forward audit on one artifact whose active path contains
Hamming copy/type reads, exact opening/reply memory and point-key fallback.
The control is deterministic and synthetic because the private trained
full-path store is not readable from this host. Record that limitation
explicitly rather than calling it trained-model evidence.

## Cancel probe

Search the audit tests for one case that simultaneously exercises Hamming,
memory and the point key and asserts all of their active manifest fields and
stage costs. If it exists, stop. Separate Hamming and memory tests are not a
substitute.

## Scope

- `crates/uor-r4-cpu-train/src/served_forward.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-8/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

- A deterministic compact artifact activates Hamming copy/type reads at stream
  and readout depths, opening/reply memory, a point table and the native head.
- The audit compares every Q12/token/kept-cell decision with
  `IntegerStackSession` and reports the active mechanisms and nonzero
  Hamming/memory cost counters.
- At least one run exercises point fallback after an opening miss; a control
  without the point table records the different head result.
- The test emits a stable decision hash and deterministic report.
- Tests, formatting and CPU Clippy pass. A trained full-path artifact remains
  an explicit limitation unless credentials become available.
