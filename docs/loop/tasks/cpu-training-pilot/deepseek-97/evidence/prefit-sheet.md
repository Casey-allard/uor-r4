# CPU final served-gain distance training

**Status:** READY FOR FIT REVIEW

TEST FITNESS: PENDING

## Question

Can the final FBS normalization gain be trained directly by complete served
target distance on two public model seeds, improve held-out repeated-key
predictions over frozen weights and an equal-move random gain control, and
retain exact CPU/serving parity?

This is CPU development research under D84. It is not a GPU qualification,
language-capability result or complete-reply result.

## Base artifacts

- seed 1:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2:
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

The source artifacts contain no Hamming readout. The trainer must fail closed
when the final-query-dependent Hamming readout is present.

## Source and binary

- source commit: `ef75026b4`;
- driver:
  `local/deepseek/served-gain-target/release/cpu-gain-panel-driver`, SHA-256
  `f115b3d49d5bc9b709128e8d7ca0c67aa9c8fa2cb8c81a2e3cf8ca9b738ff553`;
- Rust/Cargo: `1.97.1`;
- host: AMD Ryzen 9 9950X, Ubuntu 24.04.3 LTS, 16 physical cores and 32
  logical CPUs; the registered runs use 16 workers, one per physical core.

## Data split

Reuse the deepseek-95 repeated-key panel exactly: 58 even-document training
examples and 56 odd-document evaluation examples from eight shared eight-token
keys. Both seeds use the same selection, source order, key list and random
control seed.

## Preconditioning and arms

The same eight cached memory-weight updates from deepseek-95 first produce the
frozen gain baseline on every seed. Their pinned final hashes are:

- seed 1:
  `3ad89c7b80df52f9e1e4d878c072982e9fb613b8453ded0dd50c3ee905e0ee44`;
- seed 2:
  `070d66364112d4527d3043998f023f435ec73455b9a617aa94695403c62bd089`.

The comparison then holds the memory tables fixed:

| Arm | Gain |
|---|---|
| frozen | eight-step memory baseline, no gain update |
| trained | eight one-code distance updates to `final_norm_gain` |
| random | same number of uniformly selected ±1 gain-code moves |
| memory-off | source artifact with memory disabled |

Each training step evaluates every final-gain coordinate at ±1 code with the
complete writer objective. Ties retain the first candidate in coordinate and
sign order. Independent candidates run through the 16-thread Rayon pool.

## Optimization

At each teacher-forced position the cached path stores:

- the exact normalization immediately before `final_norm_gain`;
- the fixed memory hit, table identity and stored token;
- the fixed source sample boundary for checked aggregation.

A candidate applies the same gain grid to the cached normalization, applies
the fixed Q24 memory movement in the normalization-fine domain, converts to
Q12, runs the writer and recomputes point fallback when an opening key misses.
The input token sequence and recurrent state never change.

The full-forward trainer remains available behind `--reference`.

## Commands

```sh
CARGO_TARGET_DIR=local/deepseek/served-gain-target \
  cargo build --locked --package uor-r4-cpu-train --release \
    --bin cpu-gain-panel-driver

RAYON_NUM_THREADS=16 cpu-gain-panel-driver ARTIFACT TOKENS MASK MANIFEST OUT \
  --expected-artifact-sha256 ARTIFACT_SHA \
  --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
  --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
  --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
  --expected-memory-sha256 MEMORY_TRAINED_SHA \
  --memory-steps 8 --steps 8 --keys 8 --per-key 8 --key-tokens 8 \
  --reply-turn 8 --seed 1
```

The driver must refuse any selection other than eight keys, 58 training
samples and 56 evaluation samples. It reports a pass or negative verdict and
checks the elapsed/RSS stops after each candidate sweep. It claims a fresh
output directory before reading the model, writes no existing output leaf, and
seals and verifies a complete success or failure receipt.

## Minimum test power

Two public artifact seeds, eight memory-preconditioning steps, eight gain steps,
512 exact gain candidates per step, the full 58/56 repeated-key panel, four
matched evaluation arms plus a trained-memory-off parity arm, and complete
CPU/serving parity. Synthetic equivalence must cover no-memory-hit rows,
nonzero opening and reply memory movement, point fallback, checked overflow and
fail-closed Hamming behavior.

## Bar

- **Instrument:** both runs complete; the preconditioned memory hashes equal
  the pinned values above; every accepted gain step and every held-out arm has
  zero CPU/serving query, token and kept-cell mismatches.
- **Training activity:** each seed accepts at least one of eight gain steps;
  the random arm performs exactly the accepted number of moves.
- **Matched baseline:** trained target distance is strictly below both frozen
  and random on each seed, and trained top-1 is not below either control.
- **Ranking:** the paired top-1 gain above frozen must exceed
  `3 * sqrt(b + c)`, where `b` and `c` are the discordant frozen-only and
  trained-only rows. This threshold is part of the pass bar.
- A byte, reading or parity difference is an instrument failure. A distance or
  ranking miss is a negative for this bounded gain configuration.

On a pass, retain the trained artifact as the candidate for the next unseen-key
or complete-reply evaluation and extend the CPU trainer suite with this final
gain group. On a miss, close the one-code final-gain configuration and move to
the next distinct training-through-serving group; do not rerun this
configuration at another seed count without a new sheet.

## Cost and stop

Two local Ryzen 9 9950X runs on Ubuntu 24.04.3 LTS, 16 Rayon workers, at most
20 minutes each. Stop after a candidate sweep if it exceeds 15 minutes, after
the full run if it exceeds 20 minutes, or immediately after any observation of
peak RSS above 1 GiB. No GPU is used.

## Claim boundary

A pass establishes a positive repeatable final-gain distance result on this
public repeated-key panel with exact CPU/serving parity. It does not prove
unseen-key transfer, language quality, complete replies or GPU qualification.

## Verdict

TEST FITNESS: PENDING
