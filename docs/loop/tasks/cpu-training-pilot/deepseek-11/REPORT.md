# REPORT: bounded CPU counterfactual parameter comparison, deepseek-11

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base:
`a36d258276496b7d38c763fea532133c961e7d71`.

## What did not work, first

- **No parameter group produced a replicated public ranking gain.** Top-1
  stayed exactly at baseline for normalization, bias and decay on both seeds.
  Top-8 also stayed at baseline except seed-1 normalization, which fell by one
  row. The required rises were 43/47 rows on seed 1 and 40/44 on seed 2.
- **Route updates overfit the training samples and sharply degraded the
  held-out panel.** Seed 1 fell from 278/384/396 to 132/194/204
  top-1/top-8/admitted; seed 2 fell from 224/285/292 to 191/249/259.
- **Normalization lost admission on seed 1.** It changed 278/384/396 to
  278/383/394, so the registered gate rejected it despite a lower training
  margin.
- **Decay accepted no update on either seed.** Positive candidate work was
  present, so this is a preserved configuration negative rather than an
  instrument pass.
- **Bias and normalization changed panel margin without changing ranking.**
  Bias improved the held-out margin sum by 196 Q12 units on seed 1 and 22 on
  seed 2. Normalization improved seed 2 by 64,664 but worsened seed 1 by
  45,200 and lost two admissions there.

## What changed

Added `cpu-counterfactual-corpus-driver`, a Rust runner that joins the strict
masked-corpus split to the existing exact `CpuCounterfactualTrainer` and
`CpuPanelEvaluator`.

One process runs one named arm:

- `baseline` executes the corpus and panel with zero updates;
- `normalization`, `bias`, `decay` and `route` each run 128 deterministic
  BVG-K steps with four coordinates, delta 1 and margin Q12 1;
- outputs use create-new files and cannot overwrite earlier evidence; and
- final checkpoints reload to the same artifact and step count before the
  result is emitted.

## Measurements

All values are over the same 1,024-row document-disjoint public evaluation.

| Seed | Arm | Accepted | Top-1 | Top-8 | Admitted | Gate |
|---:|---|---:|---:|---:|---:|---|
| 1 | baseline | 0 | 278 | 384 | 396 | control |
| 1 | normalization | 3 | 278 | 383 | 394 | reject admission |
| 1 | bias | 2 | 278 | 384 | 396 | accept, rank unchanged |
| 1 | decay | 0 | 278 | 384 | 396 | no-op |
| 1 | route | 11 | 132 | 194 | 204 | reject admission |
| 2 | baseline | 0 | 224 | 285 | 292 | control |
| 2 | normalization | 2 | 224 | 285 | 292 | accept, rank unchanged |
| 2 | bias | 1 | 224 | 285 | 292 | accept, rank unchanged |
| 2 | decay | 0 | 224 | 285 | 292 | no-op |
| 2 | route | 7 | 191 | 249 | 259 | reject admission |

Instrument:

- 10/10 processes exited 0 with empty stderr.
- Every arm checked 262,144 Q12 query coordinates, 1,024 tokens and 8,192
  kept cells with zero mismatches.
- Both zero-update artifacts were byte-identical to their pinned sources.
- Every trained checkpoint reloaded to its final artifact and step count.
- Total process elapsed was 586.137 seconds; peak RSS was 69,616 KiB.

The registered verdict is **negative** for a replicated public ranking gain
under all four tested 128-step, four-coordinate, delta-one parameter groups.
This closes those configurations, not their mechanism families.

Full machine-readable record:
[comparison.json](evidence/comparison.json). Raw outputs remain under
`local/deepseek/cpu-counterfactual-compare-2026-11-02/`.

## Checks run at the head

- Runner unit test in a fresh target: 1 passed.
- Full CPU release library suite plus runner test in a fresh target:
  126 passed, 0 failed, 4 ignored; runner test 1 passed.
- CPU strict Clippy with `--all-targets --no-deps -- -D warnings`: passed.
- Targeted `rustfmt --check`: passed.
- FIT/PRE-RUN review and `prep-check`: FIT, complete, `may_compute=true`.
- Preflight `sha256sum -c`: all 9 source/binary/input pins passed.
- Run postflight: 10/10 arms complete, all per-arm stderr empty, output
  `SHA256SUMS` verified.
- `git diff --check`: passed.

## Skills used

No available Codex skill applied. The existing counterfactual trainer, corpus
loader, panel evaluator, project rules and prior pilot were read directly.

## Still open

- Other BVG-K budgets, coordinate blocks, step sizes and mixed parameter groups
  remain untested.
- A genuinely fresh complete-reply panel and GPU-qualified run remain
  unavailable.
- No useful language, fresh-phrasing or GPU-equivalence claim follows.

## Plain summary

The CPU can update non-token model parameters through exact counterfactual
steps and preserve every served decision. On these public seeds, however, those
updates either did nothing measurable, lost admission, or overfit and harmed
the held-out panel. This is useful evidence about where CPU training is not yet
working, not a new chat result.
