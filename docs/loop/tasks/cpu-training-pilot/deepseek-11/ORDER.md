# ORDER: cpu-training-pilot, deepseek-11

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `a36d258276496b7d38c763fea532133c961e7d71`.

## Goal

Run a materially different bounded CPU learning comparison. The prior
rank-margin pilot updated tied token-table coordinates and failed its ranking
bar. This unit tests lawful BVG-K counterfactual updates to non-token FBS
parameter groups against matched zero-update controls.

## Scope

- `crates/uor-r4-cpu-train/Cargo.toml`
- `crates/uor-r4-cpu-train/src/bin/cpu-counterfactual-corpus-driver.rs`
- `docs/harness/prep/cpu-training-pilot-counterfactual-compare-2026-11-02.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-11/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Required design

- Add a Rust-first runner over the existing strict masked-corpus split,
  `CpuCounterfactualTrainer` and `CpuPanelEvaluator`.
- Support one arm per process: zero-update baseline, normalization, bias, decay
  or route.
- Use a fixed BVG-K coordinate block and fixed update budget before compute.
- Record accepted/refused and joint/single updates, candidate attempts, final
  artifact/checkpoint hashes, exact CPU/serving panel parity, admission and raw
  top-1/top-8/margin/rank readings.
- Preserve existing rank-margin code and defaults.
- Write only under the caller's fresh output directory; refuse to overwrite an
  existing final artifact.

## Done when

- Parser, baseline/control and one accepted group update pass a finite test.
- The full CPU library suite and CPU-local strict Clippy pass.
- A FIT/PRE-RUN-PASS sheet is written before the public comparison.
- Both public seeds run the fixed zero-update and four parameter-group arms
  over the same 1,024-row document-disjoint panel.
- Every arm preserves admission where applicable and has exact CPU/serving
  export, or the run records its registered failure.
- REPORT and notebook preserve the verdict, including a negative.
- A fresh exact-head review approves the PR.
