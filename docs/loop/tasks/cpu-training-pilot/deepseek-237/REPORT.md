# REPORT: cpu-training-pilot, deepseek-237

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[readout-refit sheet](../../../../harness/prep/cpu-training-pilot-readout-refit-2026-11-06.md).
Base: `1f2bd76ce`.

## What did not work, first

Complete as **REJECT**. On both read-free public FBS artifacts, the
coordinate-wise median refit lowered held-out top-8. Seed 1 also lowered top-1;
seed 2 gained 39 top-1 rows but still lost 17 top-8 rows. The registered
two-artifact bar therefore failed.

This closes the tested eight-observation coordinate-wise median and
mean-centroid output-row refits for the read-free raw-FBS/shared-table
configuration. It does not reject the geometric writer, a different output
objective or a future two-table artifact.

## What changed

The run captured the exact Q12 query before every masked TRAIN response target,
then compared:

1. the original shared table;
2. a decoupled identical table;
3. coordinate-wise median output rows;
4. coordinate-wise mean-centroid output rows;
5. a matched cyclic target-label shuffle;
6. the shared-table median diagnostic.

Only output rows for targets with at least eight TRAIN observations changed.
The input table, SpiralCore/FBS trunk and writer-cell admission path remained
fixed for the primary arms. The pinned artifacts declare `read=dot` and contain
no Hamming or turn-memory tables, so this result makes no Hamming-path claim.

## Measurements

Fresh panel:
`local/deepseek/readout-refit/2026-11-06/final-2789/panels/dd-2789`,
seed 2789, `dev,dev`, protocol 2, 128 conversations and 5,072 masked targets.

- tokens:
  `48e3d45303ab303e8e32460cb123ca183ac378822ebb5d8ab59f535e12d991f0`;
- mask:
  `42175a3e2c00e4c1b5ef59ab8e270969ae4331e8456bb5900817d02a026d8bc6`;
- manifest:
  `7ac81f888d2a440905981d6ca115b3c6ef6b817f9c80675e72feaa751fc57027`.

TRAIN expanded to 5,114 targets. The floor of eight observations retained 146
rows and 4,142 observations per artifact. The matched shuffle changed 4,127
assignments. Independent writer scoring reproduced every captured TRAIN and
panel decision with zero changes.

| Seed | Arm | Top-1 | Top-5 | Top-8 | Admitted |
|---:|---|---:|---:|---:|---:|
| 1 | original | 1,539 | 1,955 | 2,037 | 2,106 |
| 1 | output median | 1,355 | 1,640 | 1,678 | 2,106 |
| 1 | mean centroid | 1,442 | 1,782 | 1,831 | 2,106 |
| 1 | shuffled target | 207 | 527 | 556 | 2,106 |
| 1 | shared median diagnostic | 148 | 287 | 302 | 832 |
| 2 | original | 1,239 | 1,445 | 1,472 | 1,524 |
| 2 | output median | 1,278 | 1,438 | 1,455 | 1,524 |
| 2 | mean centroid | 1,283 | 1,447 | 1,466 | 1,524 |
| 2 | shuffled target | 323 | 783 | 807 | 1,524 |
| 2 | shared median diagnostic | 233 | 342 | 362 | 552 |

The median versus original paired comparison was
`208` candidate-only / `392` control-only on seed 1 and
`120` / `81` on seed 2. Median beat the shuffled control on both seeds, but
the primary bar also required no top-8 loss and success on both artifacts.

The shared-table diagnostic changed 4,695/4,569 panel decisions and fell to
148/233 top-1. Output fitting is materially less destructive when the input
path is decoupled, but the tested output rows still fail the registered bar.

Both fresh-process repeats matched after timing removal:

- seed 1 normalized hash:
  `98807cb77186beac5def74fb254de3e29bfe8840ee12cb637f0a186246fc6c5e`;
- seed 2 normalized hash:
  `2a68dfd24c8e6c1d26223fa8bee49a1c3052a8d497343f19fbc50d0c4d18649c`.

The aggregate deterministic core is
`f8be6e2f6b92a6983759bb25c6f4142b16a8dbcf00695f8937c79ba26f5c009c`.

Resources:

- total measured wall: **110 s**;
- TRAIN preflight: **5.45 / 5.42 s**, 67,308 / 67,504 KiB peak RSS;
- panel generation: **0.59 s**, 7,892 KiB peak RSS;
- seed 1 / seed 2: **16.91 / 16.71 s**, 88,308 / 88,956 KiB peak RSS;
- seed 1 / seed 2 repeat: **16.95 / 16.72 s**;
- retained evidence: **133,677 bytes**;
- post-seal free storage: **287,695,409,152 bytes**;
- sealed `SHA256SUMS`:
  `fd3b3e675821ebcebc5a55d837fb82e549d37db958d4996cd77d2aa25a28edd1`.

Compact evidence: [result.json](evidence/result.json).

## Execution deviation

The first transaction block generated and pinned the fresh panel, then stopped
before creating reports because the registered `mkdir "$REPORTS"` assumed its
`reports/` parent existed. The panel was not regenerated and no model,
threshold, shuffle or comparison was changed. The same pinned panel was
resumed after creating the missing parent. The deviation is recorded in the
sealed `execution-note.txt`.

## Interpretation

Coordinate-wise output-row centering is not a stable readout improvement for
these artifacts. It moves substantial geometry and loses top-8 on both seeds;
seed 1 also loses top-1. The seed-2 top-1 gain is real under the paired rule but
does not survive the complete registered bar.

The large shared-table regression confirms that changing the input path while
reusing that table as output is harmful. That observation supports keeping
input and output roles explicit, but it does not authorize the tested median
rows as a serving change.

## Checks run at the head

- TEST FITNESS: FIT and D67 pre-run GO were recorded before seed 2789 opened;
- execution used clean reviewed head `8907ce5c9` and release binary
  `714047ba4c4670e52ae1cf2d1d9f68bc1feef7f66feae80c52a0b336bf89c071`;
- 10 driver/aggregation tests and 425 CPU library tests passed; four release
  benchmarks were ignored;
- scoped Clippy, new-file Rustfmt, shell syntax and JSON checks passed;
- the typed aggregator rejects contradictory status/outcome reports and turns
  malformed inputs into `FINAL_CONTROL_FAULT`;
- input, source, generator, tokenizer and artifact pins passed;
- both artifacts, both fresh-process repeats, panel identity and normalized
  determinism passed;
- the resource limits and post-seal storage floor passed;
- the evidence seal verified.

## Skills used

No system-provided Codex skill applied. Repository pre-registration, FIT/GO,
D84 CPU research and evidence-sealing procedures were used.

## Still open

- a different output objective that preserves top-8, such as a rank-margin or
  port-aware two-table update rather than coordinate centering;
- an artifact/export interface that can represent distinct input and output
  tables without silently changing serving geometry;
- a new fresh panel and pre-registration for that objective; seed 2789 is
  consumed and must not be reused.

## Plain summary

Refitting only the output rows did not improve the model consistently. It lost
top-8 on both public seeds, so the registered experiment is a rejection for
coordinate-wise median and mean-centroid output rows. The much worse shared
table diagnostic still suggests that input and output roles should stay
separate, but the next attempt needs a rank-aware objective and a new panel.
