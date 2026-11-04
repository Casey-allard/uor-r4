# ORDER: cpu-training-pilot, deepseek-104 (unseen-key point transfer)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `2789caa90`.

## Goal

Run the missing transfer test on the complete delivered CPU path. The
deepseek-95 and deepseek-97 public runs reused exact opening keys between
training and evaluation. This unit derives a point-key table from training
examples, evaluates it on document-disjoint examples whose exact opening keys
never occur in training, and compares point-on, point-off and a matched random
target table on both public FBS model seeds.

## Design

- Select training groups from even document ordinals and evaluation groups from
  odd ordinals. The two sets of first-eight-token opening keys are disjoint.
- Derive one opening/reply memory image only from training examples. Confirm
  every evaluation opening key is absent from the served memory table.
- Derive the point table from training prompts by stepping each prompt through
  the exact CPU FBS trunk and storing its final Q12 query with the first reply
  token as target. The table is sorted deterministically.
- Compare three artifacts: memory with no point table, the training-derived
  point table, and the same points with deterministically shuffled targets.
- Score each evaluation example on opening top-1, opening target distance and
  full teacher-forced top-1/distance. Run the complete served-forward audit for
  every arm.
- Run both public model seeds with the same selected keys, examples, point rows
  and random seed. CPU execution uses a bounded Rayon pool; ordered reductions
  preserve exact results.

## Tests first

1. Add a finite test showing an unseen exact memory key misses while the point
   table names its nearest training target.
2. Show shuffled targets change the named opening without changing point rows.
3. Show the driver refuses overlapping train/evaluation keys, an exact memory
   hit on evaluation, malformed point rows, non-deterministic ordering and any
   CPU/serving mismatch.

## Pre-registration

Write `docs/harness/prep/cpu-training-pilot-point-transfer-2026-11-03.md`
before the two public runs. The bar requires:

- instrument: exact parity, no evaluation exact-key hit, identical selected
  inputs across seeds;
- on both seeds, point-on opening top-1 strictly above point-off and random;
- point fallback changes only the served token, so opening distance must be
  identical across point-off, point and random; a difference is an instrument
  fault;
- report the paired three-sigma opening-ranking threshold separately.

## Done when

1. Both seeds complete and every arm has zero CPU/serving query, token or
   kept-cell mismatches.
2. The pre-registered transfer verdict is recorded without tuning after seeing
   results.
3. Package tests, strict Clippy, formatting, evidence checksums, loop report,
   repository gates, exact-head review and protected merge pass.

## Cancel probes

- The public artifacts and corpus inputs are present locally and pinned by the
  deepseek-95 FIT sheet.
- Current `main` has exported points, point-key insertion, complete CPU forward
  and served-forward audit.
- Claude's open #3629 changes only float scoring defaults and records; this unit
  does not edit those files.

## Stop and report

Stop on key overlap, an exact-memory evaluation hit, point/query shape mismatch,
non-deterministic table construction, parity loss or a public-input hash
mismatch. Do not tune the point table, key count, examples or bar after
observing a result. This remains CPU development research, not GPU
qualification or a language-capability claim.
