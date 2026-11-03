# ORDER: cpu-training-pilot, deepseek-98 (parallel panel and counterfactual scoring)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `c701525932b9c5247cbe44658900414ff3ce030c`.

## Goal

Use the workstation's independent cores for the two remaining serial CPU
training/evaluation paths without changing any integer decision:

1. evaluate independent panel rows through a bounded Rayon pool and combine
   their integer readings in source order;
2. score independent counterfactual candidates in parallel while preserving
   the existing coordinate, lane and joint-selection order.

The online update sequence remains serial because each accepted update changes
the next query/artifact.

## Cancel probe

At the base, `CpuPanelEvaluator::evaluate` walks every sample serially
(`crates/uor-r4-cpu-train/src/panel.rs:173`) and
`CpuCounterfactualTrainer::train_step` scores coordinates and lanes serially
(`crates/uor-r4-cpu-train/src/counterfactual.rs:194`). The release panel
benchmark measured 10,836 rows/s on one thread. The prior counterfactual
benchmark measured about 758-790 candidate evaluations/s.

The automated claim failed before creating a branch because `loop claim`
generated a commit without the required `Lab:` and `Order:` trailers; it
reported a clean rollback. This ORDER was created by hand from fresh
`origin/main`, with the same task identity and no overlapping open order.

## Files in scope

- `crates/uor-r4-cpu-train/src/panel.rs`
- `crates/uor-r4-cpu-train/src/train.rs`
- `crates/uor-r4-cpu-train/src/counterfactual.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-panel-export-bench.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-counterfactual-bench.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-98/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. Panel rows are independent parallel tasks, return to their original slots,
   and reduce through checked integer arithmetic in source order.
2. Counterfactual candidates are independent parallel tasks; best-lane,
   coordinate, eligible-set and joint decisions are identical to the serial
   implementation at every tested worker count.
3. Tests compare serial and parallel readings, accepted artifacts, checkpoints
   and the next dependent training step exactly.
4. Release benchmarks report worker count, stable checksums, elapsed time and
   measured speedup for 1, 8, 16 and 32 workers on this host.
5. The package release tests, strict Clippy and formatting pass.
6. Notebook, program page and REPORT classify the result as an instrument or
   performance result, not a language or capability gain.
7. Exact-head review, PR checks, merge and fresh-main verification pass.

## Stop if

- Thread scheduling changes any candidate tie, accepted coordinate, artifact
  byte, panel reading or next training step.
- Parallel execution requires shared mutable evaluator state or weakens the
  complete CPU-versus-serving parity check.
- The bounded fixture shows no wall-time benefit at 16 physical workers.
- The unit would edit Claude's open files or change a registered capability
  bar.
