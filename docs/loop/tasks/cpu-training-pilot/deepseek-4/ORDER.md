# ORDER: bounded CPU discrete corpus driver, deepseek-4

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `b1e377b18`.

## Goal

Add a bounded CPU corpus-level driver over the existing exact discrete read, proposal, chooser, apply, BVG-K and checkpoint machinery so discrete candidate updates can be screened locally without touching Claude's float/FBS serving path.

## Cancel probe

Inspect the CPU discrete checkpoint driver and source corpus loaders first. If a runnable corpus driver already exists, bind it and add only missing parity/receipts. Do not create a second trainer if the exact proposal path cannot supply a real update.

## Scope

- `crates/uor-r4-cpu-train/**`
- `Cargo.toml`, `Cargo.lock`
- `docs/loop/tasks/cpu-training-pilot/deepseek-4/**`
- `docs/research/notebook/cpu-training.md`
- `docs/research/cpu-training-program.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No edit to `crates/uor-r4-discrete`, `uor-r4-float-train` or `uor-r4-integer`. No GPU run, paid compute or capability claim.

## Done when

- A Rust CLI accepts an exact discrete artifact, canonical `UORT` tokens, matching mask, manifest, seed, bounded step count, evaluation stride/cap and checkpoint interval/directory.
- Training examples are document-disjoint and use only causal positions authorized by the response mask.
- The driver uses the existing exact discrete proposal/apply machinery rather than a synthetic loss substitute.
- It writes hash-bound interval/final checkpoints and emits a reloadable discrete artifact or explicitly records why none changed.
- Receipts report accepted/refused updates, admission, top-1/top-8, exact export parity, throughput and wall/RSS cost.
- A resumed process produces byte-identical final checkpoint/artifact/panel state to an uninterrupted finite fixture, or the limitation is recorded without claiming equivalence.
- Focused tests, CPU crate suite, Clippy, formatting and roadmap/harness checks pass.
- REPORT and notebook classify the result as a component/build result or tested configuration, never a language result.

## Stop

Stop on read/proposal parity loss, admission loss, non-exact export, nondeterministic resume or unavailable public artifact/corpus identity. A tooling failure is not a mechanism negative.
