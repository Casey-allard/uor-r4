# REPORT: cpu-training-pilot, deepseek-183

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `7787313b2`; implementation pin:
`722abaff3`.

## Status

Complete as a deterministic two-artifact **REJECT** for eight-step query-port
root training. Exact parity held, but the trained arm did not clear both
seeds.

## What did not work

Seed 1 met the admission/top-1 safety bar and beat the table-anchor baseline
and random control on held-out port distance. It still lost to the
shuffled-cell control by **3,055** units:

- trained: **25,331,024**;
- shuffled cells: **25,327,969**;
- random: **25,480,346**;
- table-anchor baseline: **25,493,709**.

Seed 2 reduced port distance below all three controls, but lost one admitted
target and one top-1 against the table-anchor baseline:

- trained: **28 admitted / 9 top-1 / 19 top-8**, distance **27,312,048**;
- table anchor: **29 / 10 / 20**, distance **27,634,674**;
- random: **28 / 9 / 19**, distance **27,618,635**;
- shuffled cells: **28 / 9 / 19**, distance **27,443,854**.

The combined table-anchor intervention was much more damaging than the
query-side training recovered. Against raw frozen, admitted/top-1/top-8 fell:

- seed 1: **245/181/236 → 120/55/86**;
- seed 2: **171/136/164 → 29/10/20**.

Both seeds accepted all eight moves, matched shuffled-control accepted counts,
and had zero query, token or kept-cell parity mismatches. This is a mechanism
negative for the tested objective and an important warning about changing the
shared input/writer table's ports. It does not separate changed recurrent
inputs from direct writer-row mismatch, and it does not reject geometric
admission.

## What changed

- Added `CpuPortRootTrainer`, a cached exact final-gain replay trainer whose
  objective is the Q12 L1 distance from each target query's two 8-coordinate
  ports to that target cell's roots.
- Restricted trainable final-gain coordinates to `0..8` and `W-8..W`.
- Added safety gates against training admission, top-1 and top-8 regressions,
  with complete CPU/serving parity required after accepted moves.
- Added a deterministic table-port anchoring projection and read-back audit.
- Added matched shuffled-cell and random-gain controls plus distinct
  `NO_MOVE`, `CONTROL_FAULT`, `INSTRUMENT_FAULT`, `REJECT` and `KEEP`
  classifications.
- Added `cpu-query-port-driver` and a pre-registered two-seed run.

No serving, CUDA, float-training, exporter or Claude-owned source changed.

## Measurements

- FIT head: `e2e756dee`; GO head: `944a7fb4e`;
- training power: **235** conversations / **2,708** response rows;
- panel power: **58** conversations / **616** response rows;
- input train/dev overlap: **0** identical tokenized documents;
- accepted moves: **8/8** on each seed;
- shuffled accepted / attempts: **8/8** on each seed;
- seed 1 wall/RSS: **66.166 s / 156,920 KiB**;
- seed 2 wall/RSS: **66.726 s / 156,612 KiB**;
- all five arms on both seeds had **0** query-coordinate, token and kept-cell
  mismatches over **157,696** coordinates, **616** decisions and **4,928**
  kept cells per seed;
- in-process artifact and step replay reproduced exactly on both seeds.
- PR #3736 reviewed head `56057171e`, protected merge
  `60fb4b0bd8a08c93896fb209775117c9dc16fb20`.

Compact evidence:
[query-port-summary.json](evidence/query-port-summary.json).

## Checks run at the head

- focused port and control tests;
- full CPU package: **347 library tests passed**, 4 expected ignores, all
  binary tests passed;
- strict Clippy, rustfmt and `git diff --check`;
- FIT review and D67 pre-run GO;
- both registered 16-worker runs;
- sealed report and manifest verification from the driver;
- exact parity over every arm and both artifact seeds.
- exact-head review, all required CI checks and the protected merge queue;
- fresh-main CPU library **347 passed**, 4 expected ignores;
- fresh-main CPU binaries **104 passed**;
- fresh-main strict Clippy, report/evidence, rustfmt and `git diff --check`;
- fresh-main `loop gates --full`: **11 / 11 green**.

## Size

The unit adds one trainer family, one driver, focused tests and compact
evidence. No model artifact or corpus is committed.

## Skills used

No system-provided Codex skill applied. Repository preregistration, compute
admission, evidence and review procedures were used.

## Still open

The result argues against replacing the shared input/writer table's port blocks
without controlling their dual effect. A later unit should separate writer
addressing from input embedding, or train the projected ports jointly with an
input-code preservation objective. Do not retune this exact eight-step
configuration on these panels. Claude's actual anchored training artifact
should still be measured for the same combined table-anchor cost.

## Plain summary

Training only the query's 16 port-scaling gain codes can move those ports
toward the right roots, but it did not win both matched controls. On one seed
the shuffled-root control was slightly better; on the other, the trained arm
lost one admitted token and one top-1. More importantly, pinning every token
table row's ports to roots sharply damaged ordinary writer decoding. That
treatment changes both recurrent inputs and writer rows, so this run does not
say which effect dominates. The next geometric path needs to preserve learned
token identity while adding cell-address structure.
