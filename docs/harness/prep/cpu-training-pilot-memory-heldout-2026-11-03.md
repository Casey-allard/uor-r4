# CPU repeated-key memory-weight comparison

**Status:** READY FOR FIT REVIEW

TEST FITNESS: FIT

## Question

When the same exact opening key appears in training and evaluation documents,
does eight full-path memory-weight updates improve held-out next-token distance
and top-1 beyond frozen zero weights and an equal-count random-move control,
without losing exact CPU/serving parity?

This is CPU development research under D84. It is not a GPU qualification, a
language-capability result, or evidence that CPU training replaces Claude's GPU
main track.

## Base artifacts

Both model seeds use the same corpus-derived memory tables and key split:

- seed 1:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`, SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`, SHA-256
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- DEV tokens:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-tokens.u16`, SHA-256
  `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
- response mask:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-response-mask.u8`, SHA-256
  `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`;
- manifest:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-manifest.json`, SHA-256
  `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab`.

The source manifest says `split=train`; base-model exposure is unverified.
Evaluation documents are document-disjoint from training documents but their
opening keys intentionally overlap.

## Data recovery and split

For every contiguous `response_mask == 1` span:

- the reply is that complete span, which includes its terminating EOS;
- the prompt is the current turn after the latest BOS or EOS boundary;
- if prompt plus reply exceeds context 192, the prompt is truncated from the
  left to fit; replies of 192 or more tokens are excluded;
- examples retain their source document ordinal.

Even document ordinals are training; odd ordinals are evaluation. Group by the
first 8 prompt tokens. Keep groups with at least four examples on each side,
then order by:

1. `min(training_count, evaluation_count)`, descending;
2. `evaluation_count`, descending;
3. `training_count`, descending;
4. key bytes lexicographically descending.

Select the first eight groups and cap each side at eight examples in source
order. The resulting 58 training and 56 evaluation examples are fixed before
the model run. The selected keys are:

```text
[201, 55, 2728, 28, 439, 969, 2461, 14]
[201, 55, 2728, 28, 439, 275, 351, 261]
[55, 2728, 28, 1049, 970, 1171, 2655, 28]
[201, 55, 2728, 28, 1044, 1788, 329, 435]
[55, 2728, 28, 772, 529, 91, 2655, 1795]
[201, 55, 2728, 28, 340, 1003, 275, 69]
[201, 55, 2728, 28, 1049, 970, 1171, 14]
[201, 55, 2728, 28, 273, 540, 519, 579]
```

## Memory tables

Derive one opening row per selected key from the most common training reply's
first token, with ties to the smaller token id. Derive reply rows from the
training prompt/reply sequences at key window 192, reply turn limit 8, and the
documented reply key shape. Both movement-weight rows start at zero.

Attach the same derived memory image to each artifact seed. Existing artifact
bytes must remain unchanged; the new tables are appended and read back before
training.

## Arms

Every arm uses the same attached memory image and the same 56 evaluation
examples:

| Arm | Weights | Memory signal |
|---|---|---|
| frozen | all zero | on |
| trained | eight coordinate steps | on |
| random | same number of accepted moves as trained | on |
| memory-off | attached artifact | off |

The trained arm tries every opening/reply weight coordinate at positive and
negative one Q24 unit (`0.0625`), with a `1.0` bidirectional bound. It commits
the first strictly best coordinate, where top-1 is considered before lower
aggregate target distance. Trainer seed is `1`. Independent candidate
artifacts are evaluated in parallel across the host's Rayon worker pool, while
ordered chunk reduction keeps the accepted coordinate deterministic.

The random arm uses seed `1`, makes one uniformly selected Q24 move in the
opening or existing reply table per accepted trained move, and records every
move.

## Commands

## Steps

Build once and pin the binary:

```sh
CARGO_TARGET_DIR=local/deepseek/memory-heldout-parallel-target \
  cargo build --release --locked -p uor-r4-cpu-train \
    --bin cpu-memory-panel-driver
```

Run each seed with the hashes above:

```sh
cpu-memory-panel-driver ARTIFACT TOKENS MASK MANIFEST OUT \
  --expected-artifact-sha256 ARTIFACT_SHA \
  --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
  --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
  --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
  --steps 8 --keys 8 --per-key 8 --key-tokens 8 --reply-turn 8 --seed 1
```

1. Build the release driver once and record source, binary, Rust, hardware,
   memory and filesystem identity.
2. Verify all four input hashes for each model seed.
3. Run seed 1 and compare its one-step projection against the 30-minute stop.
4. Complete seed 1, then run seed 2 with the identical selected keys.
5. Preserve every arm artifact and report, compare the two seeds and write the
   final evidence receipt.

## Bar

- **Instrument:** both runs finish; the source hashes, selected keys, training
  and evaluation membership and derived memory are identical across seeds;
  attached tables read back exactly; every artifact arm has zero CPU/serving
  query, token and kept-cell mismatches over all 56 held-out examples.
- **Training activity:** each seed accepts at least one of the eight steps and
  the random control makes exactly that many moves.
- **Matched baseline:** frozen and memory-off have identical top-1 and distance
  readings because zero weights make no movement and no-signal memory does not
  apply.
- **Dense held-out improvement:** on each seed, trained
  `target_distance_sum` is strictly below frozen and random, while trained
  top-1 is not below either control.
- **Ranking signal:** on each seed and for each control arm, trained top-1
  exceeds that control's top-1 by at least
  `ceil(3 * sqrt(N * p * (1 - p)))`, where `N` is the full decision count and
  `p = control_top1 / N`. The frozen and random controls must both be cleared.
  Trained top-8 is unavailable in this first unit.
- **Positive:** all instrument and dense checks pass on both seeds. The ranking
  signal is reported separately as positive only if both seeds also clear the
  three-sigma top-1 threshold.
- Anything else is a bounded configuration negative. Do not tune steps, step
  size, keys, caps or seed after observing results.

## Minimum test power

Two published model seeds, eight selected repeated-key groups, 58 training
examples and 56 evaluation examples per seed. Shared keys make the memory
decision reachable; the document split prevents using held-out examples in the
table or weight objective.

## Cost and stop

Local Ryzen 9 9950X only: 16 physical CPUs used through Rayon, no GPU, pod,
rental, upload or paid API. The two runs may write roughly 13 MiB of artifacts
plus reports. The first seed is stopped before eight steps if its one-step
projection would put the second seed beyond 30 minutes total. A started
eight-step run is not shortened because its readings are unimpressive.

Stop on any identity mismatch, missing key group, malformed prompt/reply, memory
read-back mismatch, non-finite reading, failed reload, lost parity, or random
control with a different accepted-move count.

## Claim boundary

A positive dense result says this bounded repeated-key panel benefits from
served memory-weight updates. It is not evidence of transfer to unseen keys,
complete-reply quality, fresh phrasing or GPU-qualified training. A negative
closes this key set, eight-step schedule, Q24 amplitude and public
internal-development panel.

## Verdict

TEST FITNESS: FIT - M2 #2030
sheet: docs/harness/prep/cpu-training-pilot-memory-heldout-2026-11-03.md
reviewer: deepseek fresh local adversarial review
note: sheet SHA-256
`b7ece95523a88a22259f743daa0ea6823feefeb92f7ea1ceed4b03b09adcb269`;
repeated-key split is reachable and document-disjoint; 58 training and 56
evaluation examples; both public seeds, frozen/trained/random/memory-off
controls, eight Q24 steps, deterministic ordered parallel candidate reduction,
exact parity, dense and three-sigma ranking bars fixed before compute;
30-minute stop
date: 2026-11-03

A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone
before compute.
