# ORDER: CPU served-forward contract and audit, deepseek-5

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `0fa4c5fce`.

## Goal

Define and exercise a canonical exact forward for the supported server-valued FBS mode. Add an independent CPU audit that records the model mechanism manifest, compares complete decisions with `IntegerStackSession`, identifies float-only training operations, and reports stage timing, modelled table bytes, candidate comparisons and throughput. This is a development instrument, not a GPU qualification or language result.

Claude owns PR #3558 and the float/integer training and serving files. This unit does not edit them and does not become a dependency of Claude's active work.

## Cancel probe

Inspect `uor-r4-cpu-train`, `fbs-export`, `IntegerStackSession`, and `train-in-space --train-as-served`. Stop if an existing command already emits exact per-stage decisions, CPU/serving parity, a mechanism manifest, float-only operation inventory, table bytes and throughput for the same artifact. Otherwise this unit adds the missing audit in `uor-r4-cpu-train`.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-5/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No edits to `crates/uor-r4-float-train`, `crates/uor-r4-integer`, or `docs/labs/one-model-end-to-end/**`.

## Done when

- A library type defines the supported exact FBS contract: representation, operation order, rounding, saturation, tie rules, active reads, memory and readout.
- A CPU command accepts a public FBS artifact and token sequence and emits a deterministic JSON report with artifact identity and active mechanism manifest.
- Every CPU query, token and kept-cell decision is compared exactly with `IntegerStackSession`; any mismatch is a row-0 instrument fault.
- The report counts modelled candidate comparisons and table bytes by stage without presenting arithmetic counts as measured wall time.
- The report measures CPU stage latency and throughput on the named Ryzen 9 9950X.
- The report lists operations still performed by the float training forward that are absent from the exact artifact forward and assigns each a required gradient treatment: exact, straight-through, secant, or unresolved.
- Focused tests cover malformed artifacts, unsupported modes, deterministic output, stage/byte accounting and planted parity failure.
- Public two-seed or equivalent controls run where fixtures are available; evidence, README and REPORT state actual results and limitations.

## Stop and ask if

- the audit requires changing a Claude-owned served kernel;
- exact CPU parity requires implementing an unsupported pointer, non-recurrence or unexported read;
- the work would make Claude wait for a merge or artifact from this branch.
