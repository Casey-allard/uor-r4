# ORDER: cpu-training-pilot, deepseek-221 (nearest-rival served-gain training)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `4fde31439`.

## Goal

Test the CPU analogue of Claude's successful `nearest` training objective
through the exact served FBS query path. Deepseek-97 trained the final gain by
target distance and moved no held-out top-1 row. This unit instead trains the
target's Q12 hinge against its eight nearest rival writer rows, with the rival
set refreshed after each accepted gain move.

## Fixed construction

Start from deepseek-97's frozen memory-preconditioned gain baselines for the
two public FBS seeds. Keep its 58 teacher-forced training samples and 56
held-out samples from eight repeated keys. Make eight final-gain moves per
seed.

At each step:

1. replay the current artifact to each training query;
2. choose the eight lowest-L1 writer rows other than the target;
3. score every final-gain coordinate at `+1` and `-1`;
4. minimize the sum of `max(0, 1 + target_distance - rival_distance)` over all
   decisions and eight rivals;
5. break ties by lower target distance, then more top-1 decisions.

The eight rivals are fixed while all 512 candidates for one step are scored.
This is a bounded training-through-serving surrogate because candidate gains
change the query but not the recurrent trajectory or memory decision.

Arms:

- frozen deepseek-97 gain baseline;
- nearest-rival trained final gain;
- equal-move random final gain;
- nearest-rival training against a deterministic cyclic shuffle of target
  labels;
- memory-off and trained-memory-off diagnostics.

## Data and power

- public FBS seeds 1 and 2;
- deepseek-97's 58 training and 56 held-out samples, 2,431 held-out decisions
  per seed;
- eight refreshes of the eight-rival sets per seed;
- one fresh-process repeat of seed 1 for determinism;
- local CPU only, with 24 Rayon workers.

## Bar

On both artifacts, the true-target arm must:

- accept at least one of eight gain moves and change the artifact in only the
  final-gain table;
- achieve strictly lower held-out multi-rival hinge sum than frozen, random
  and shuffled-target controls;
- improve paired top-1 over frozen by more than `3 * sqrt(b + c)`;
- retain at least the top-1 of random and shuffled-target controls;
- retain exact CPU/serving query, token and kept-cell parity on every accepted
  step and held-out arm;
- repeat byte-identically after timing removal.

## Stop rule

Stop on input drift, unavailable frozen baselines, a vacuous target shuffle,
failed parity, nondeterminism, artifact bytes outside `final_norm_gain`, any
float or dense path, exceeding 30 minutes or 2 GiB, or a failed control.

## Cost

Two public artifacts plus one process repeat; local Ryzen 9 9950X CPU,
24 workers, at most 30 minutes each, 2 GiB RSS and 64 MiB evidence.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-221/ORDER.md
