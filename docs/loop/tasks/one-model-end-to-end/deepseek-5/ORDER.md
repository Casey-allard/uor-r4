# ORDER: matched served-training comparison, deepseek-5

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `9c9b712a4`.

## Goal

Add an explicit served-forward override for checkpoint inspection and row
scoring and pre-register one bounded matched comparison of ordinary training
and full served-surrogate training. The comparison is two training modes at two
fixed seeds: four runs total, not four model mechanisms. This host
deliverable makes the comparison executable; its GPU execution is a separately
gated follow-up.

## Cancel probe

If `--inspect`/`--score-rows` can already force the served forward for a run
whose `config.json` says `train_as_served: false`, and the existing data and pod
make the four runs available, reuse it and skip the code change. Otherwise add
only the override required for a like-for-like evaluation.

## Scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/harness/prep/one-model-end-to-end-served-comparison-2026-11-02.md`
- `docs/loop/tasks/one-model-end-to-end/deepseek-5/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

- A documented `--served-forward 0|1` scoring override applies only to
  `--inspect`, `--decode-speed` and `--score-rows`; it does not alter training.
- Host controls pin the override selection and reject it outside those modes.
- The pre-registration is FIT and names a runnable same-mode evaluation-copy
  procedure, data pins, cost, stop rules and the four-run evidence required on
  a GPU.
- The report states the four GPU runs as `NOT_RUN` on this host and leaves them
  to the registered follow-up; it makes no training or capability claim.
- Tests, formatting, policy, exact-head review, merge and fresh-main verification
  follow the standard delivery path.

## Stop and ask if

- The override changes training behavior or requires a serving-format change.
- The pod exceeds the sheet's one-GPU-hour/$1.50 ceiling.
- The data pins differ from the sheet.
