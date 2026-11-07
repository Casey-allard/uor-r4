# REPORT: one-model-end-to-end, deepseek-20

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no GPU, pod, CUDA execution or spend.

## What did not work, first

- **This unit does not run the reply-masked SFT experiment.** It builds the opt-in initialization
  path that the experiment needs. Whether fine-tuning moves `served-reply-panel` remains unread.
- **Checkpoint restore is deliberately narrow.** The trunk and Hamming read load exactly; Flock,
  content, opening, prompt-register and turn-memory sidecars are refused rather than recreated
  from fresh parameters. The current `(b64L)` target needs only the Hamming read.
- **Optimizer state and the schedule are not resumed.** A resumed interrupted run would need a
  separate contract and tests; this path always starts both fresh.
- **Strict package Clippy is not green at this base.** Five pre-existing lints in
  `hamilton_sheet.rs`, `flock_read_memory.rs`, `flock_read_register.rs`,
  `flock_read_hamming.rs` and `spiralcore_fbs_sheet.rs` stop `-D warnings`. Ordinary package
  Clippy completes, and it emits none from the added initialization code.
- **Workspace formatting was already inconsistent.** `cargo fmt -p uor-r4-float-train --
  --check` reports a large diff in unchanged code. Formatting the package would bury this unit,
  so only the changed files were kept and `git diff --check` is the applicable whitespace check.
- **The first fresh review rejected the initial implementation.** It found that an
  identity-transport control checkpoint could load under the active one-atom arm because the
  source run config's `fbs_arm` was neither checked nor bound, and a partially spun Hamming
  sidecar could pass the boolean check. Both now fail closed.
- **The final fresh review found a source-config omission.** A checkpoint copied without a
  declared read sidecar could still pass when the command's read flag was zero. The
  initialization path now parses the source run config, refuses every unsupported nonzero
  read declaration and reply memory, and requires its declared Hamming head count to equal
  the command. A count-free refusal replaces the old hard-coded four-head hint.
- **The next reviews found that forward-mode flags could still change.** The source's
  `retain_rates`, `fold_scores`, `reflect_floor` and `train_as_served` values must now
  equal the command. This is material for `(b64L)`, whose source sets `retain_rates` and
  `train_as_served` true; loading it with defaults would silently read its recurrence
  table differently. `fix_input_table` is deliberately not bound because it changes
  training only, never the loaded checkpoint's forward.

## What changed

- Added `LongConfig::init_from: Option<PathBuf>`, skipped when absent. A run without
  `--init-from` therefore writes the prior `config.json` and takes the existing fresh-build path.
- Added `train-in-space --init-from CHECKPOINT`. It is accepted only for training with `--out`;
  probe, scoring, grading and recall modes refuse it before model work.
- The checkpoint path uses `StackModel::load`, verifies width, geometric architecture, six
  recurrent layers, Hamilton/geometric gates, quaternion transport, vocabulary, heads, MLP width,
  margin objective, absence of historical read state, and the shared vector head. It then applies
  the requested `--context` with `set_context`.
- `start_hamming` now loads `hamming_read.safetensors` for checkpoint runs and checks head count,
  width, copy-key mode and type spin. The loaded variables are exact; only a new optimizer is
  created. A missing required sidecar or mismatched saved read is refused.
- The source run config is required and must name `OneAtomActive`; a copied step without it is
  refused. Its path and SHA-256 are bound in `init.json`.
- A Hamming sidecar carrying `step_rotation` tensors on only some depths is refused.
- `out/init.json` uses schema `uor-r4.train-in-space-init/1`. Only checkpoint runs write it, so
  the no-flag output tree is unchanged. It records the canonical checkpoint path, source run
  config and SHA-256 of `model.safetensors`, model `config.json` and each loaded sidecar, with
  `optimizer: fresh` and `schedule: fresh`.
- Added six focused library tests and one CLI mode test. Size: about 700 added source/test lines and 4
  changed lines across the three Rust files, plus this task record and its claim.

## Measurements

No model step, forward or training pass was run. A temporary local smoke test loaded Claude-301's
seed-1 `(b64L)` step-16,384 checkpoint at width 256 and its four-head Hamming read on CPU, then
wrote and removed its provenance record. It passed. The source files observed there were:

| File | SHA-256 |
|---|---|
| `model.safetensors` | `6fd2d764171cc416d5ff272e19e3ebe0644183ab9c43ca0662706756ac75f5de` |
| `config.json` | `1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2` |
| `hamming_read.safetensors` | `672de97fcc329b10c3cde591f66047352851f9fe1d02df85a168190f5ba12ddd` |

The installed tests prove stronger value identity on generated checkpoints: every trunk tensor and
every Hamming tensor compare equal by `f32::to_bits`. The external `(b64L)` smoke proves format and
configuration compatibility but does not retain or compare all source values across a process.

## Checks run at the head

- `CARGO_TARGET_DIR=/dev/shm/deepseek-train-init-target cargo test -p uor-r4-float-train
  --lib --offline --locked`: 305 passed, 0 failed.
- `... cargo test -p uor-r4-float-train --offline initialization`: focused library and binary tests
  passed; the five library tests cover absent-flag serialization, exact trunk load and context,
  exact Hamming load and provenance, missing sidecar, unsupported read/sidecar, shape mismatch and
  fresh/checkpoint records. The Hamming module separately refuses a partially spun sidecar.
- `... cargo check -p uor-r4-float-train --all-targets --offline`: passed.
- `train-in-space --init-from /tmp/checkpoint --inspect /tmp/run`: exited 1 with
  `"--init-from applies to training with --out only"`, before a model or GPU was opened.
- `... cargo clippy -p uor-r4-float-train --all-targets --no-deps --offline`: passed with the
  pre-existing warnings listed above; no warning was emitted from the new code.
- `... cargo clippy -p uor-r4-float-train --all-targets --no-deps --offline -- -D warnings`:
  failed on the pre-existing lints, not on an added-line lint.
- `git diff --check`: passed.
- `cargo fmt -p uor-r4-float-train -- --check`: failed on the package's existing formatting drift;
  no workspace-wide formatting rewrite was kept.

Not run: CUDA tests, GPU checkpoint load, any training step, export, serving or SFT evaluation.

## Skills used

No system-provided Codex skill applied. The repository's loop, charter, geometry and compute rules
were used directly.

## Still open

- Run reply-masked SFT from `(b64L)` under a prep sheet, FIT and pre-run review, then judge it with
  `served-reply-panel`.
- Add exact restore for other reads only when their experiment needs it.
- Local branch integration, exact-head review, PR creation and delivery remain with the parent.

## Plain summary

Training can now start from Claude-301's current model instead of rebuilding it. `--init-from`
loads the model and its Hamming read without changing their values, checks that the run describes
the same architecture, starts a new optimizer and schedule, and records exactly which files were
loaded and their hashes. The existing no-flag path is unchanged, and unsupported reads fail clearly
instead of quietly receiving new random weights.
