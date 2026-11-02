# ORDER: cpu-training-pilot, deepseek-10

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `3063b20c7d1f8a6a75d52cf56675a86193b20c1e`.

## Goal

Close the train-to-audit gap for one full-path artifact. An accepted CPU
rank-margin update must preserve the artifact's Hamming, opening/reply memory
and point-key records, and the updated artifact must pass the exact
CPU-versus-serving served-forward audit with those mechanisms active.

## Scope

- `crates/uor-r4-cpu-train/src/train.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-10/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Required design

- Build one deterministic synthetic `/8` fixture with Hamming copy/type reads,
  opening memory, two reply-memory rows, a point-key fallback and the native
  cell-writer head.
- Train its writer table through the existing exact `CpuRankMarginTrainer`
  path with at least one accepted update.
- Require every byte outside `writer.table` to remain identical and require the
  trained artifact to parse.
- Run `audit_served_forward` on both the untrained and trained artifacts over
  the same fixed token sequence and reply cursor.
- Require exact query-coordinate, token and kept-cell parity on both audits;
  preserve Hamming, opening/reply memory and point-table manifests after the
  update; require non-zero mechanism costs.
- Prove the trained writer table changes a complete served decision relative
  to the untrained control, or record that the selected fixture cannot do so
  and repair the fixture rather than weakening the audit.
- Keep the result explicitly synthetic. It covers the full served path on an
  artifact with a trained writer table; it is not trained Hamming/memory
  weights, a public-model result, useful chat or GPU qualification.

## Done when

- The focused integration test passes.
- The full `uor-r4-cpu-train` library suite passes.
- Targeted formatting and `git diff --check` pass.
- The REPORT and CPU-training notebook record the exact result and boundary.
- A fresh exact-head review approves the PR.
