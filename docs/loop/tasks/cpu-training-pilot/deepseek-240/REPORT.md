# REPORT: cpu-training-pilot, deepseek-240

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506),
supporting
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Base: `d43e33950`.

## What did not work, first

- The first checkpoint-mode smoke omitted `--raw-gates --raw-transition`.
  The exporter correctly refused the train-as-served b64L run instead of
  silently exporting snapped arithmetic. The rerun with both raw flags
  reproduced both accepted artifact hashes.
- The bounded complete-reply arm did not demonstrate useful conversation. All
  32 generated replies, 16 per artifact, hit the token cap; none emitted BOS and
  none matched its exact reference. All 32 were non-empty, but the artifacts had
  only 11 and 14 distinct complete sequences and 10 and 8 replies repeated a
  4-gram. This remains a readiness negative rather than a new model result.
- Fresh-context review found that the shared reply generator always used the
  writer head. On nearest-serving b64L artifacts that generated non-served
  replies. The generator now selects the artifact's declared head, and a
  regression requires nearest and writer choices to differ while the complete
  reply follows nearest.
- Nearest-readout artifacts have no public top-8/target-admission API matching
  the native writer's. This report therefore records top-1 for nearest and all
  four readings only for native-writer artifacts.
- The independent served-forward audit covers one 192-token window per artifact.
  Panel evaluation covers every selected row, but it does not trace every panel
  prefix through the independent audit.
- The current native panel evaluator accepts the training vocabulary of 4,096.
  A future native artifact at another valid serving vocabulary needs a wider
  panel evaluator before this command can grade it.

## What changed

- Added `cpu-checkpoint-grade`, one Rust command with two modes:
  - grade an existing `UORLUT01` artifact;
  - export a saved checkpoint through the CPU-only path and grade the result.
- Added deterministic bounded causal-row and complete-conversation sampling to
  the masked-corpus API. Selection scans the corpus in source order at evenly
  spaced ranks.
- Added a grading report bound to the artifact, corpus, manifest, tokenizer,
  deterministic sample fingerprint and optional baseline identity.
- Nearest heads use the served whole-table nearest decision. Native writer
  artifacts use the existing exact `CpuPanelEvaluator` for top-1, top-8 and
  target admission.
- The report always includes an independent CPU-versus-serving audit over one
  context window and bounded complete-reply diagnostics.
- Baseline grading uses the same selected rows and reply examples. Nearest
  reports candidate-only and baseline-only top-1 row counts; native reports
  top-8 and admission deltas.
- Existing outputs and duplicate output paths are refused before checkpoint
  export. Input files and corpus identity are checked before export as well.

No float trainer, CUDA source, serving kernel or Claude-owned path changed.

## Measurements

The registered development panel was `dd-2788`:

| Input | Identity |
|---|---|
| split | `dev` |
| tokens | 15,512 |
| eligible response targets | 4,900 |
| token SHA-256 | `1d049e5e3c0a95cceb5a45e5acd569a64bc17c3087d59f1c1aca66880c2bde3d` |
| mask SHA-256 | `8b79a29c5bf27616e2337bdf05b444364f94e7d07acd3e144dc7714e96c9fe68` |
| tokenizer SHA-256 | `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377` |

The full run selected all 4,900 eligible rows and 16 complete conversations:

| Seed | top-1 | Distinct non-empty | BOS complete | Cap stopped | Exact reference |
|---:|---:|---:|---:|---:|---:|
| 1 | 1,036 / 4,900 | 11 / 16 | 0 / 16 | 16 / 16 | 0 / 16 |
| 2 | 1,110 / 4,900 | 14 / 16 | 0 / 16 | 16 / 16 | 0 / 16 |

Seed 2 against seed 1 on the identical rows:

- net top-1: **+74**;
- seed 2 only: **181**;
- seed 1 only: **107**.

Both artifacts passed the audit:

| Reading | Result |
|---|---:|
| positions | 192 |
| Q12 query coordinates | 49,152 |
| query mismatches | 0 |
| token decisions | 192 |
| token mismatches | 0 |
| kept cells | 1,536 |
| kept-cell mismatches | 0 |

Checkpoint mode reproduced the accepted artifacts:

| Seed | Artifact | Snapshot |
|---:|---|---|
| 1 | `9a370566cd9da163191c0196763c68ddb395927fcf61fdecc1736b04941a4bd9` | `0c2f4c1cefa293c5076293535e3dda4b206e409167610733d75ea3455bdf722c` |
| 2 | `5039cadefd60633bcff5ac8511aff362e1174ef4cda20844a8febad625d2e304` | `a80c406da9ace065d90d78aeaf9115b4c937b182f5b72b11fe900368b31f8f6d` |

The elapsed full seed-1 reports were 8.99 s and 6.56 s on an otherwise active
host. Seed 2 plus its seed-1 baseline took 15.26 s. Timings include corpus
sampling, row grading, the audit and bounded reply generation; they are not a
benchmark.

After removing wall and rate fields, the two full seed-1 reports both normalized
to SHA-256
`98d0b7a8a362c28d58d5c925b7f605767e8512f34ad0eb146af0eda44821803e`.
Their selected reply inputs had fingerprint
`7da9a28e3ea6cca22d3bf570764454d700bfabcce7df03cb450033b85214c196`.

The compact bound result is
[result.json](./evidence/result.json).
Raw reports and ignore-only exported artifacts are under
`local/deepseek/checkpoint-grade/2026-11-07/`.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-cpu-train --no-fail-fast`:
  **all library, binary and doc tests passed**; the library ran
  **459 passed, 4 ignored, 0 failed**.
- focused checkpoint-grade tests: **3 passed**.
- focused corpus sampling test: **1 passed**.
- CLI argument tests: **3 passed**.
- release build of `cpu-checkpoint-grade`: passed.
- checkpoint-mode export of both b64L seeds reproduced the accepted artifact and
  snapshot hashes.
- full seed-1 fresh-process normalized replay: identical.
- `cargo clippy --release --locked -p uor-r4-cpu-train --all-targets --no-deps`:
  no warning in the new files; only the crate's pre-existing warning categories
  remain.
- `rustfmt --check` on both new Rust files and `git diff --check`: passed.

## Skills used

No system-provided Codex skill applied. The repository's D84 CPU research,
loop, evidence and delivery rules were used directly.

## Still open

- A generic nearest top-8 viewer if a future selected artifact needs it.
- Native-writer grading above or below the training vocabulary's 4,096 rows.
- Per-layer or all-prefix CPU/serving localization when the single-window audit
  finds a mismatch.
- Float-to-integer complete-reply equivalence. This command grades the integer
  artifact; it does not run the float model.
- Model quality. Both b64L seeds remain cap-stopped, exact-reference-negative
  development baselines, not useful chat models.

## Plain summary

There is now one CPU command that can take a saved geometric checkpoint,
convert it to the serving artifact without CUDA, evaluate a large held-out
panel, generate bounded replies, compare two artifacts on identical rows, and
prove that the independent CPU forward matches serving. On the accepted b64L
pair it reproduces both known exports and checks all 4,900 rows and 32 complete
replies. The result also confirms that these models still do not converse well:
every sampled reply hit its limit and none matched its reference.

References #3506, #3407.
