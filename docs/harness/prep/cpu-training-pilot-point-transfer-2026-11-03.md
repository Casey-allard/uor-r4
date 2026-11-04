# CPU unseen-key point transfer

**Status:** FIT

TEST FITNESS: FIT

## Question

Can a point table derived from training-only prompt queries name the first
reply token on document-disjoint prompts whose exact eight-token opening keys
never occur in training, beyond point-off and matched random-target controls,
while preserving exact CPU/serving parity on both public model seeds?

This is CPU development research under D84. It is not GPU qualification or a
language-capability result.

## Base artifacts

Both model seeds use the same public corpus and selected key split:

- seed 1 artifact:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2 artifact:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- DEV tokens:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-tokens.u16`,
  SHA-256
  `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
- response mask:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-response-mask.u8`,
  SHA-256
  `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`;
- manifest:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-manifest.json`,
  SHA-256
  `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab`.

The source manifest says `split=train`; base-model exposure is unverified.

## Fixed split

Use the existing conversation recovery: each contiguous masked response is a
reply including its EOS, and its current turn is the prompt. Even document
ordinals are training; odd ordinals are evaluation.

Group by the first eight prompt tokens. Rank training groups by example count
then descending key bytes and keep eight, capped at eight examples each. Rank
evaluation groups with keys absent from every selected training key by the
same rule and keep eight. Selection-only was run before this sheet and fixed:

- 60 training examples across 8 keys;
- 60 evaluation examples across 8 keys;
- zero exact opening-key overlap.

Training keys:

```text
[55,2728,28,772,529,91,2655,1795]
[55,2728,28,1049,970,1171,2655,28]
[55,2728,28,2237,2693,2655,601,28]
[201,55,2728,28,273,540,519,579]
[201,55,2728,28,340,1003,275,69]
[201,55,2728,28,439,275,351,261]
[201,55,2728,28,439,969,2461,14]
[201,55,2728,28,1049,970,1171,14]
```

Evaluation keys:

```text
[201,55,2728,28,1997,351,3,201]
[201,55,2728,28,1201,351,1203,579]
[201,55,2728,28,1044,1788,329,435]
[201,55,2728,28,273,540,519,265]
[55,2728,28,1049,612,71,299,571]
[201,55,2728,28,1143,351,1125,311]
[55,2728,28,318,2301,3,201,35]
[201,55,2728,28,2577,664,2295,1139]
```

## Derived memory and point table

Derive opening/reply memory only from the 60 training examples at key window
192, reply turn 8 and movement weight zero. Confirm every evaluation opening key
misses the exact opening table.

Build one point row per distinct training prompt query through the production
point-key builder:

- call `build_point_key` on each training prompt window;
- do not mark reply start, so no training-time memory move enters the point;
- record the pre-memory trunk Q12 query at the prompt's final row;
- target the first reply token;
- coalesce identical queries by commonest target, ties to the smaller id;
- sort rows lexicographically by point.

The random control uses the same points with targets permuted by a deterministic
Fisher-Yates shuffle at seed 1. If that draw is unchanged, rotate the target
rows once; if the control is still unchanged because every target is equal,
refuse the run rather than score a vacuous control.

**Instrument correction before rerun.** The first two public runs used a custom
query helper that marked reply start while deriving training points. A fresh
exact-head review found that this captured a memory-moved query, unlike the
production `build_point_key`. Those runs are retained as invalid for production
point-transfer evidence. The split, dimensions, point rows, controls and bar
are unchanged; only the query construction now calls the production builder.

## Arms

Every arm carries the same derived memory and evaluates the same 60 held-out
samples:

| Arm | Point table |
|---|---|
| point-off | absent |
| point | training-derived |
| random | same points, shuffled targets |

The opening decision is the prompt-final served step after memory lookup and
point fallback. Report opening top-1 and target distance. Also report the full
teacher-forced top-1 and distance from the existing exact evaluator.

## Bar

- **Instrument:** all three arms finish on both seeds; evaluation has no exact
  memory-key hit; 60 evaluation examples and 60 decisions per arm; every arm
  has zero CPU/serving query-coordinate, token and kept-cell mismatches.
- **Point effect on both seeds:** point opening top-1 is strictly above
  point-off and random.
- **Distance invariant:** point fallback changes only the served token, not the
  final query, so all three arms must have identical opening target distance.
  A difference is an instrument fault, not an improvement.
- **Random control:** random and point-off are both reported, whether or not
  they tie.
- **Ranking signal:** on each seed report point top-1 minus point-off top-1
  against `ceil(3 * sqrt(N*p*(1-p)))`, where `N = 60` and `p` is point-off
  top-1 / 60. This is a separate positive only if cleared on both seeds.
- Anything else is a bounded unseen-key negative. Do not tune keys, rows,
  targets, table size, random seed or scoring after observing results.

## Steps

Build once in a fresh target:

```sh
CARGO_TARGET_DIR=target/deepseek/unseen-point-build-104 \
  cargo build --release --locked -p uor-r4-cpu-train \
    --bin cpu-point-transfer-driver
```

Run each seed with the pinned hashes and fixed dimensions:

```sh
cpu-point-transfer-driver ARTIFACT TOKENS MASK MANIFEST OUT \
  --expected-artifact-sha256 ARTIFACT_SHA \
  --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
  --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
  --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
  --train-keys 8 --evaluation-keys 8 --per-key 8 \
  --key-tokens 8 --reply-turn 8 --random-seed 1
```

## Minimum test power

Two public model seeds are required. Each seed uses the same 8 training keys
and 8 disjoint evaluation keys, 60 training examples and 60 evaluation
examples. The 60 held-out openings are the ranking unit; the full
teacher-forced check adds 2,050 decisions per arm. Point, point-off and random
must all run on each seed, and the strict-above-both bar must hold on both.
This is the minimum bounded power for this exploratory transfer probe and does
not qualify language capability.

## Cost and stop

Local Ryzen 9 9950X only; Rayon uses the host pool, with no GPU, pod, rental,
upload or paid API. Inputs and outputs are small. Stop on any hash, selection,
key-disjointness, point-shape, exact-key-hit, deterministic-table, artifact
reload or CPU/serving parity failure.

## Claim boundary

A positive says the finite training-derived point table generalizes to the
fixed unseen exact keys on this public internal-development corpus and both
public model seeds. It does not establish complete replies, fresh reply
wording, useful chat or GPU-qualified training.

## Verdict

TEST FITNESS: FIT - M2 #2030
sheet: docs/harness/prep/cpu-training-pilot-point-transfer-2026-11-03.md
reviewer: deepseek fresh local adversarial review, round 3
note: 60 training / 60 evaluation examples; 8 training and 8 evaluation
opening keys with zero overlap; training-only point queries; point-off,
point and non-vacuous shuffled-target controls; exact CPU/serving parity;
opening top-1 plus separate three-sigma rule; distance equality is an
instrument invariant; five focused tests pass
date: 2026-11-03

A FIT verdict clears compute for the configuration in that sheet only (D22).
