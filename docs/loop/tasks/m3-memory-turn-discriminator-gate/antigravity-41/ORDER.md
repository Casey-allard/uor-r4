# ORDER: m3-memory-turn-discriminator-gate, antigravity-41

Lab: antigravity. Item: [m3-memory-turn-discriminator-gate](../../../../roadmap/items/m3-memory-turn-discriminator-gate.md).

## Task

Address the exact-head review findings on PR `#3079` (`1ddfa1cbabb066ffa2c19cb27e61e476ebbefb35`, `antigravity-40`):
1. **Restore `#[cfg(any())]` on `Softmax`, `SparseSoftmax`, `RelationRoute`, `SpanHead`, and `SavedCompiler` (CHARTER §3.2, §3.3, §3.4):** Put `#[cfg(any())]` back on `Softmax`, `SparseSoftmax`, `RelationRoute`, `SpanHead`, `HeadParts`, `DenseParts`, `CombinedParts`, `CompilerArtifact`, `SavedCompiler`, `encode`, and `impl TurnCompiler for SavedCompiler` in [`crates/uor-r4-training/src/relation_compiler.rs`](../../../../../crates/uor-r4-training/src/relation_compiler.rs), and remove the softmax-head `logits_milli` / `margin_milli` methods so zero legacy `Softmax`/`SparseSoftmax` or dense `f64` dot-product code is compiled or executed under `cargo test`.
2. **Add a compiled, 100% integer-only `GatedTurnCompiler<C>` wrapper and state the margin wiring status plainly:** Add `GatedTurnCompiler<C>` in `crates/uor-r4-training/src/relation_compiler.rs` implementing `TurnCompiler` with a configurable `MemoryTurnGate` (`new`, `with_memory_turn_gate`, `set_memory_turn_gate`, `memory_turn_gate`) and optional integer `StatementMargin` callback (`with_margin_fn`), test it end-to-end (`gated_turn_compiler_wires_configurable_gate_and_integer_margin` and `memory_turn_gate_achieves_zero_false_writes_on_frozen_nonrelation_prose_panel`), and state plainly in `REPORT.md`, `README.md`, and `preregistration-DRAFT-2026-10-23.md` that (a) `SavedCompiler` and its legacy float heads stay compiled out behind `#[cfg(any())]` with `None` margin until a lawful integer geometric head replaces them, and (b) `milestone_world_v2`'s 32 Stage-A tests and `stack_tracking`'s Stage-A code remain compiled out behind `#[cfg(any())]` as on `main`.

## Files and bounds

- `crates/uor-r4-training/src/relation_compiler.rs` — restore `#[cfg(any())]` on legacy heads/`SavedCompiler`, add `GatedTurnCompiler`, update tests
- `docs/labs/m3-memory-turn-discriminator-gate/README.md` — state `#[cfg(any())]` and integer margin status plainly
- `docs/labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md` — state `GatedTurnCompiler` and margin status plainly
- `docs/loop/tasks/m3-memory-turn-discriminator-gate/antigravity-41/ORDER.md` — this repair order
- `docs/loop/tasks/m3-memory-turn-discriminator-gate/antigravity-41/REPORT.md` — task report

## Done when

- `Softmax`, `SparseSoftmax`, `RelationRoute`, `SpanHead`, and `SavedCompiler` are back behind `#[cfg(any())]`.
- `cargo test -p uor-r4-training --lib` passes (`53 / 53` tests, including `79 / 79` generator templates from `milestone_world_v2::relations()`, `0 / 50` false writes through `GatedTurnCompiler`, and configurable gate + integer margin controls).
- `uor-harness ratchet d23`, `d25`, `d32`, `loop report --check`, `loop pr-check --local`, and `loop gates --full` pass.

## Cancel probe

- Probe: `uor-harness loop gates --quick` (cancelled with SIGINT)
- Cancel signal: `SIGINT` (`exit 130`), clean exit in `< 1s`.
