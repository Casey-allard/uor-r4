# ORDER: one-model-end-to-end, deepseek-20

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `b80933c4b`.

## Goal

**Start a training run from an existing checkpoint instead of a fresh seed.** The immediate
experiment is reply-masked SFT from Claude-301's `(b64L)` step-16,384 model. Without this unit,
that experiment must repeat the full base-model run. The opt-in `--init-from CHECKPOINT` path
loads the checkpoint's exact trunk and Hamming-read tensors, creates a fresh optimizer and
training schedule, and records the source files and their SHA-256s under `out/init.json`.

No flag keeps the current fresh initialization and `config.json`. This unit does not run the
SFT experiment or claim that fine-tuning improves replies.

## Cancel probe

Class: mechanism plumbing. At base `b80933c4b`, no source or open branch implements
`train-in-space --init-from`; `in_space_long::start_hamming` always creates fresh Hamming
parameters. Claude-310 established the capability failure this experiment will test: `(b64L)`
completed 0 of 256 held-out rows, and its DEV reply cap has 756 scored EOS targets per million
against DEV's 85,812.

## Done when

1. `--init-from` is accepted only for training with `--out`, not recall or evaluation modes.
2. With no flag, fresh model construction and serialized `config.json` are unchanged; with it,
   `StackModel::load` preserves every trunk tensor and the loaded Hamming tensors exactly.
3. Width, architecture, pattern, gates, vocabulary, head binding and configured Hamming shape
   are checked before training. A configured Hamming read requires its sidecar. Any configured
   Flock, content, opening, register or turn-memory read is refused rather than freshly created.
4. The requested training context is applied. The optimizer and schedule start fresh.
5. `out/init.json` records the mode, source paths and SHA-256 of the model, model config and
   loaded sidecars, and states that optimizer/schedule state is not resumed.
6. Focused CPU tests cover absent-flag compatibility, exact load, missing sidecar, shape/config
   mismatch, unsupported reads and CLI mode refusal; formatting, offline tests, all-target check
   and Clippy pass.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-20/**`

## Stop and ask if

Ask means decide from the repository, record it and continue.

- Exact loading needs a different saved-checkpoint format or an optimizer resume; neither is in
  scope, and neither is implied by this goal.
- A change would edit Claude's active source files, pods or PRs.

Laptop only: no pod, GPU, CUDA execution, spend or external write. Bound: one unit.
