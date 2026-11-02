# Prep sheet: bounded CPU rank-margin step comparison, 2 November 2026

Lab: deepseek. Item: [cpu-training-pilot](../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [deepseek-9](../../loop/tasks/cpu-training-pilot/deepseek-9/ORDER.md).
Source base: `9c9b712a4`.

TEST FITNESS: FIT

## Question

The published 256-attempt CPU rank-margin pilot moved each selected Q12
coordinate by one unit and did not move top-1 or top-8 on either public FBS
seed. Is that configuration limited by an update step too small to affect
ranking? On the Ryzen 9 9950X, compare the default one-Q12-unit step with
larger positive steps over the same corpus, seeds, schedule and capped panel,
while preserving admission and exact CPU/serving export.

This is a public internal-development experiment. It does not establish
language capability, fresh-phrasing transfer, complete replies or GPU
qualification.

## Base artifacts

- Seed 1: `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`.
- Seed 2: `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256 `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

Both are public raw FBS artifacts, vocabulary 4,096, context 192, native
writer, no Hamming or turn memory.

## Data split

- DEV tokens: `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-tokens.u16`, SHA-256
  `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`.
- Response mask: `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-response-mask.u8`,
  SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- Manifest: `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-manifest.json`, SHA-256
  `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab`.
- Tokenizer: `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.

The split is deterministic and document-disjoint: evaluation stride 2 selects
198 documents and is capped evenly at 1,024 rows; the remaining 200 documents
and 7,080 targets train. The public manifest marks the source as `train`;
historical model exposure is unverified. These are internal-development rows,
not sealed held-out data.

## Arms

All arms use trainer seed 1, 1,024 update attempts, checkpoints every 256,
evaluation stride 2 and a 1,024-row cap.

| Arm | Margin Q12 | Step Q12 | Parameter limit Q12 |
|---|---:|---:|---:|
| baseline | 1 | 1 | 8,192 |
| unit | 1 | 1 | 8,192 |
| step-16 | 1 | 16 | 8,192 |
| step-64 | 1 | 64 | 8,192 |

Each artifact seed gets all four arms. The baseline is a zero-update run through
the same driver and evaluator; the three training arms start from fresh loads of
the pinned artifact.

Command template:

```sh
cargo run --release --locked -p uor-r4-cpu-train --bin cpu-corpus-driver -- \
  ARTIFACT DEV_TOKENS DEV_MASK DEV_MANIFEST \
  1 STEPS 2 1024 256 RUN_DIR \
  --margin-q12 1 --step-q12 STEP_Q12 --parameter-limit-q12 8192
```

`STEPS` is 0 for baseline and 1,024 for each training arm.

## Bar

For each artifact seed, compare every training arm with that seed's own
zero-update baseline over the same 1,024 rows.

- **Instrument:** complete with finite readings, all 1,024 rows evaluated,
  baseline/final Q12 query, token and kept-cell mismatches exactly zero, final
  artifact reloadable, and checkpoint binding/reload exact.
- **Admission:** final `target_admitted` is at least baseline.
- **Ranking:** either top-1 or top-8 rises by at least
  `ceil(3 * sqrt(N * p * (1-p)))`, where `p` is that seed's baseline rate for
  the metric. The other ranking metric may not fall.
- **Update activity:** a training arm must accept at least one update.
- **Larger-step positive:** step-16 or step-64 must satisfy every row above on
  both seeds and beat the unit arm's top-1 on both seeds.
- A unit-only pass is recorded as a length effect, not evidence for the larger
  step hypothesis. No arm passing the larger-step rule is a negative for these
  bounded step settings.

## Measurements

For every arm and seed:

- accepted/refused updates and final epoch;
- top-1, top-8, target admission, margin-loss sum and target-rank sum;
- step-only and wall throughput;
- final artifact and checkpoint SHA-256;
- CPU/serving query, admitted-cell, top-8 and token parity;
- process peak RSS and machine identity.

## Steps

1. Build the exact source, record source and binary hashes, and run the focused
   driver and checkpoint-binding tests.
2. Verify all artifact and corpus hashes above.
3. Run baseline then unit, step-16 and step-64 for seed 1.
4. Repeat in the same order for seed 2.
5. Apply the bar from raw panel readings, write a SHA-256 manifest, and report
   every arm, including failures.

## Cost

Local Ryzen 9 9950X only: no GPU, pod, rental, upload or paid API. The previous
256-step run took about 66 seconds per seed including 1,024-row evaluation.
Eight runs at 1,024 attempts are projected below 30 minutes total; no external
compute cost.

## Minimum test power

Two published model seeds, 1,024 fixed document-disjoint rows per seed and three
positive-step arms. The three-standard-deviation ranking threshold is a
screening bar for development evidence, not a claim of statistical independence
from the public base model's exposure.

## Stop rule

- Stop a seed on artifact/corpus/hash mismatch, non-finite or missing reading,
  failed artifact reload, or any CPU/serving export mismatch.
- A completed arm is not shortened because its loss or ranking is unimpressive.
- Stop before 1,024 only if projected local wall time exceeds one hour for that
  arm or the trainer loses the source admission constraint.
- Keep failed artifacts and reports; do not tune arms after seeing results.

## Claim boundary

The strongest possible result is that a larger tested coordinate step improved
public document-disjoint internal ranking while preserving admission and exact
serving export on both seeds. It does not establish useful conversation,
generalization to fresh phrasing, GPU equivalence or a reason to stop GPU work.

## Verdict

TEST FITNESS: FIT - fresh non-author adversarial review of sheet SHA-256
`10944b513fcf51d1c7550be8bd2be1e759a1dc6f0e0093a0f3c1c6eb6dfba3b8`;
PRE-RUN: PASS.

The reviewer confirmed that every pinned path and hash resolves, the split is
document-disjoint, the 1/16/64-Q12 step arms are materially new, the ranking and
admission gates are fixed, the update geometry is checkpoint-bound, and the
claim remains limited to public internal-development evidence.
