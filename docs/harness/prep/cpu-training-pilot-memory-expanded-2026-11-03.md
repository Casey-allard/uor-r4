# Expanded CPU memory-weight panel

**Status:** FIT

TEST FITNESS: FIT

## Question

Does the exact training-through-serving memory-weight surrogate produce a
replicated ranking gain when the repeated-key panel is increased from 8 keys /
60 training / 56 evaluation examples to 16 keys / 102 training / 114
evaluation examples?

## Base artifact

| Input | Identity |
|---|---|
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |
| hardware | AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads |

## Data split

The driver's deterministic first-eight-token key selection gives:

- 16 keys;
- up to 16 examples per key;
- 102 training examples;
- 114 evaluation examples;
- no document split changes from the existing selector.

These counts were checked with `--select-only` before this sheet was frozen.
The selection is:

```text
training evaluation key
10       11         [201, 55, 2728, 28, 439, 969, 2461, 14]
7        10         [201, 55, 2728, 28, 439, 275, 351, 261]
7        10         [55, 2728, 28, 1049, 970, 1171, 2655, 28]
6         8         [201, 55, 2728, 28, 1044, 1788, 329, 435]
10        6         [55, 2728, 28, 772, 529, 91, 2655, 1795]
8         6         [201, 55, 2728, 28, 340, 1003, 275, 69]
7         6         [201, 55, 2728, 28, 1049, 970, 1171, 14]
7         6         [201, 55, 2728, 28, 273, 540, 519, 579]
6         6         [55, 2728, 28, 310, 75, 3, 201, 35]
5         8         [201, 55, 2728, 28, 273, 540, 519, 265]
6         5         [201, 55, 2728, 28, 1997, 85, 261, 648]
6         5         [55, 2728, 28, 935, 71, 612, 2655, 315]
5         5         [55, 2728, 28, 318, 91, 3, 201, 35]
4        12         [201, 55, 2728, 28, 1997, 351, 3, 201]
4         5         [201, 55, 2728, 28, 1044, 81, 435, 627]
4         5         [201, 55, 2728, 28, 301, 1417, 351, 2274]
```

## Steps

Build the existing release binary, record its SHA-256 and the source commit,
then run seed 1 and seed 2 sequentially with:

```text
env RAYON_NUM_THREADS=32 target/release/cpu-memory-panel-driver \
  ARTIFACT DEV_TOKENS DEV_MASK DEV_MANIFEST OUT \
  --expected-artifact-sha256 ARTIFACT_PIN \
  --expected-tokens-sha256 TOKENS_PIN \
  --expected-mask-sha256 MASK_PIN \
  --expected-manifest-sha256 MANIFEST_PIN \
  --steps 32 --keys 16 --per-key 16 --key-tokens 8 --reply-turn 8 --seed 1
```

The driver derives turn memory from training examples, caches the exact
teacher-forced trajectory, trains opening and reply Q24 movement weights, and
evaluates trained, frozen, equal-move random and memory-off arms.

## Bar

Instrument:

- fixed selection counts and hashes;
- 32 attempted steps on each seed;
- equal-move random control;
- exact CPU/serving parity for every arm;
- no input, seed, key or step change after observing results.

Candidate, on each seed:

- at least one accepted movement-coordinate update;
- trained top-1 strictly above frozen and equal-move random;
- trained target distance strictly below frozen and random;
- trained top-1 not below either control;
- trained top-1 gain over each control at least
  `ceil(3 * sqrt(N * p * (1 - p)))`, where `N` is the full decision count and
  `p = control_top1 / N`.

The primary comparison is trained versus equal-move random. Trained versus
frozen is supporting evidence. Failure of any below-threshold bar is a
configuration negative, not a rejection of memory weighting.

## Minimum test power

Two model seeds; 114 repeated-key evaluation examples; 32 exact candidate
updates; matched frozen, random and memory-off controls. Repeated keys make
this development evidence only.

## Cost and stop

Local Ryzen 9 9950X only. Cached trajectory, 32 steps, 32 workers per seed;
no GPU, pod or paid API. Run seed 1 first. Stop before seed 2 if seed 1's
measured wall projects the two-seed total beyond 30 CPU minutes. A started
32-step seed is not shortened because its readings are unimpressive. Stop on
selection drift, parity failure, nondeterministic artifact/checkpoint state or
any post-observation tuning. Record source commit, release binary SHA-256,
Rust version and host identity before the runs.

## Verdict

TEST FITNESS: FIT - M2 #2030
sheet: docs/harness/prep/cpu-training-pilot-memory-expanded-2026-11-03.md
reviewer: Codex fresh-context non-author review
note: fixed 16-key/102-training/114-evaluation selection independently
reproduced; expanded cached memory-weight surrogate; frozen, trained,
equal-move random and memory-off controls; established
ceil(3*sqrt(N*p*(1-p))) ranking bar; exact CPU/serving parity; staged
30-minute cost stop; source/binary/Rust/host identity required
date: 2026-11-03

A FIT verdict clears compute for the configuration in that sheet only (D22).
