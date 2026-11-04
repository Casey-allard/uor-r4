# CPU eight-row point neighborhood

**Status:** FIT

TEST FITNESS: FIT

## Question

Does a majority vote over the eight nearest production point rows improve
unseen-exact-key opening prediction over the single nearest row, point-off and
the same eight-row vote over shuffled targets, on both public model seeds?

This is an offline geometry screen. It does not change or claim integer serving
support for an eighth-nearest rule.

## Base artifact

Use the same pinned public artifacts and DEV token/mask/manifest as deepseek-104:

- seed 1 artifact:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2 artifact:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256 `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- tokens
  `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
- mask
  `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`;
- manifest
  `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab`.

## Data split

Keep the deepseek-104 fixed split exactly: 60 training and 60 evaluation
examples, eight training and eight evaluation first-eight-token keys, zero key
overlap, 48 production training-derived point rows, and random seed 1.

## Steps

Build once in a fresh target:

```sh
CARGO_TARGET_DIR=target/deepseek/point-neighborhood-run-105 \
  cargo build --release --locked -p uor-r4-cpu-train \
    --bin cpu-point-transfer-driver
```

Run each seed with its artifact hash and `--neighbor-vote 8`, using the token,
mask and manifest hashes above. The driver creates the memory, point and
shuffled-point artifacts, existing served opening readings, and the offline
eight-row screen in one report.

## Candidate

For each held-out opening:

1. step the prompt through the memory-carrying artifact with reply start marked;
2. confirm the exact opening key misses memory;
3. use its final Q12 query;
4. compute exact L1 distance to every point row;
5. sort by `(distance, row index)`;
6. count targets in the first eight rows;
7. choose the commonest target, ties to the smaller token id.

Also evaluate `k = 1` on the same extracted query and require it to equal the
existing served point artifact's opening token. Evaluate `k = 8` on the point
and shuffled-target tables.

## Bar

- **Instrument:** 60 openings per seed; zero key overlap; all keyed openings
  miss exact memory; extracted nearest names exactly match the existing served
  point artifact; finite controls cover wrong-nearest majority override, target
  tie break and oversized `k`.
- **Candidate on both seeds:** eight-row vote opening top-1 is strictly above
  point-off.
- **Candidate on both seeds:** eight-row vote opening top-1 is strictly above
  both the single-nearest point row and the eight-row shuffled-target vote over
  the identical points.
- Report `vote_top1 - nearest_top1` against the three-sigma threshold computed
  from the 60 nearest-row answers.
- Anything else is a bounded negative for `k = 8` on this split. Do not tune
  rows, split, distance metric or `k` after seeing the result.

## Minimum test power

Two public trained model seeds; 60 unseen-key openings per seed; fixed 48-row
point table; nearest, eight-row point, point-off and eight-row random controls.

## Cost and stop

Local Ryzen 9 9950X only; no GPU, pod or paid API. Stop on input drift, key
overlap, extraction/served mismatch, non-exact ordering or any attempt to call
the offline vote served before its separate implementation.

## Claim boundary

A positive screen justifies a separate integer serving-schema unit and audit.
It is not a language-capability, useful-chat or GPU qualification result. A
negative closes only the tested `k = 8`, exact-L1 majority configuration.

## Verdict

TEST FITNESS: FIT - M2 #2030
sheet: docs/harness/prep/cpu-training-pilot-point-neighborhood-2026-11-03.md
reviewer: deepseek fresh local adversarial review, round 2
note: fixed deepseek-104 60/60 split, 8+8 disjoint keys, 48 point rows;
exact L1 eight-row majority; served-nearest extraction check; point-off,
nearest, point-vote and shuffled-vote controls; focused 8/8; offline screen
only
date: 2026-11-03

A FIT verdict clears compute for the configuration in that sheet only (D22).
