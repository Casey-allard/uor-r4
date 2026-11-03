# CPU memory trajectory reuse equivalence and speed

**Status:** READY FOR FIT REVIEW

TEST FITNESS: FIT

## Question

Can the teacher-forced pre-memory FBS trajectory be cached once so that
opening/reply memory-weight candidate search avoids rerunning recurrence,
projection, Hamming and final normalization, while producing exactly the same
eight-step artifacts and held-out readings as the full-forward trainer, and can
it at least halve wall time on each public model seed?

This is CPU development research under D84. It is not a GPU, language or
capability result.

## Base artifacts

The reference is merged deepseek-95 at
`3b893ddc0ee29a43750632799aaae340540a0bbe`:

- seed 1 trained artifact SHA-256
  `3ad89c7b80df52f9e1e4d878c072982e9fb613b8453ded0dd50c3ee905e0ee44`,
  held-out top-1 291 and target distance 2,820,387,931;
- seed 2 trained artifact SHA-256
  `070d66364112d4527d3043998f023f435ec73455b9a617aa94695403c62bd089`,
  held-out top-1 243 and target distance 2,896,327,331.

The same public FBS artifacts, 58 training examples, 56 held-out examples,
eight opening keys, eight Q24 steps, random-control seed and hardware are used.
All source hashes and key identities remain those in the deepseek-95 FIT sheet.

## Data split

The exact deepseek-95 repeated-key panel is reused: 58 even-document training
examples and 56 odd-document evaluation examples from eight shared eight-token
keys. The source token and mask hashes, document split and key list are those
fixed in `cpu-training-pilot-memory-heldout-2026-11-03.md`.

## Optimization

For each teacher-forced position, the optimized path caches:

- the exact pre-memory normalization vector after all recurrence layers,
  projections, Hamming reads and final normalization;
- the memory hit, table identity and stored token;
- the untouched query and token decision for rows with no movement.

For a candidate weight vector, it applies only the same Q24 movement equation
to the cached normalization, converts to Q12, performs the writer decision and
applies the point fallback when the opening key missed. The fixed input token
sequence is never changed by a candidate.

The full-forward trainer remains available behind `--reference`.

## Arms

| Arm | Search | Seeds |
|---|---|---|
| reference | full forward, already retained in deepseek-95 | 1, 2 |
| optimized | cached pre-memory trajectory | 1, 2 |

Both use 16 Rayon candidate workers and the same full random/memory-off
evaluation panel.

## Steps

1. Build the optimized driver and run reference-versus-cached focused tests.
2. Run optimized seed 1 and compare all accepted steps, final artifact bytes
   and every held-out reading with deepseek-95.
3. Run optimized seed 2 under the same comparison.
4. Compute per-seed wall-time ratios and write the evidence receipt.

## Commands

```sh
RAYON_NUM_THREADS=16 cpu-memory-panel-driver INPUTS OUT \
  --steps 8 --keys 8 --per-key 8 --key-tokens 8 --reply-turn 8 --seed 1
```

The driver input arguments and expected SHA-256 flags are identical to
deepseek-95. The `--reference` flag selects the old full-forward trainer for
spot equivalence checks.

## Minimum test power

Two public artifact seeds, the full 58/56 repeated-key panel, eight accepted
steps per trained arm, four matched held-out arms and exact byte/readout
equivalence. The speed comparison uses the same host and 16 Rayon threads.

## Bar

- **Instrument:** every optimized run completes, accepts the same eight step
  receipts as the reference, and passes complete CPU/serving parity over the
  same held-out panel.
- **Artifact identity:** each optimized final artifact SHA-256 equals its
  deepseek-95 reference hash.
- **Reading identity:** frozen, trained, random and memory-off top-1 and target
  distance equal the full-forward readings exactly.
- **Speed:** each optimized run's wall time is at most half its matched
  full-forward wall time. Target-distance or ranking changes do not count as an
  optimization.
- A byte or reading difference is an instrument failure. A speed miss is a
  negative for this caching implementation.

## Cost and stop

The two optimized runs are local Ryzen 9 9950X CPU research on 16 Rayon workers.
Stop before 30 minutes. The retained full-forward controls already cost
415.370 s and 414.121 s and are not rerun unless an equivalence check fails and
needs localization.

## Claim boundary

A pass establishes exact 16-thread evaluator acceleration for the repeated-key
memory panel. It does not qualify CPU training generally, improve model
quality, test unseen keys or replace GPU qualification.

## Verdict

TEST FITNESS: FIT - M2 #2030
sheet: docs/harness/prep/cpu-training-pilot-memory-trace-2026-11-03.md
reviewer: deepseek fresh local adversarial review
note: sheet SHA-256
`ae440a491fb30badd2ed040775fb4d874e0ff9d7ab7870fc00e3488822c8a080`;
full-forward reference artifacts/readings are pinned; optimized trace keeps
exact pre-memory normalization and hit identity; two seeds, 16 threads,
byte/readout equivalence and 2x per-seed speed bar fixed before compute;
30-minute stop
date: 2026-11-03

A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone
before compute.
