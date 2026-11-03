# Served-surrogate training comparison

**Status:** EXECUTED. Instrument door PASS; operational door FAIL; quality smoke
inconclusive. The wiring is not ready for a longer run until the served-training
slowdown is repaired.

TEST FITNESS: FIT

## Question

At a fixed short budget on RTX 5090, does training through the full served
surrogate remain finite and operationally equivalent to ordinary float training,
while producing a checkpoint that export rounding cannot change? The comparison
is two modes at two seeds:

- `plain`: `--train-as-served 0`;
- `served`: `--train-as-served 1`.

This is a four-run engineering comparison. It does not establish language
capability, a milestone bar, or a preference for CPU over GPU.

## Base artifact

The base is the exact PR head reviewed before the run. It must contain
`--served-forward` and the served-surrogate training path merged through PR
#3565. The pod records:

- source SHA;
- release `train-in-space` SHA-256;
- release `fbs-snapshot` SHA-256;
- every data file's SHA-256 before the first run.

The two arms start from the same seeded initialization and use the same source
and data.

## Data split

- Data root: `/workspace/w2-inputs/2026-10-25/driver-data`.
- TRAIN tokens SHA-256:
  `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`.
- TRAIN mask SHA-256:
  `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- DEV tokens SHA-256:
  `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`.
- DEV mask SHA-256:
  `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- Tokenizer SHA-256:
  `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- Code: the exact PR head that passes the pre-run review; the pod records its SHA.
- Hardware: one RTX 5090, no more than one pod and $1.50 total.

## Recipe

The same command for both arms except the training-mode switch:

```text
train-in-space --data-root DATA --out RUN --objective cell --hamming-read 4 \
  --retain-rates 1 --decay-mult 1 --batch 16 --steps 256 --eval-every 64 \
  --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 \
  --turn-memory-dropout 500 --train-as-served MODE --seed SEED
```

Seeds are `1` and `2`. A run that writes any non-finite loss stops as an
instrument fault.

## Bar

1. **Instrument door:** all four runs complete with finite losses and load
   through the served evaluator. The served arm's saved files are byte-identical
   after export rounding; at least one plain-arm file changes.
2. **Operational door:** served training time is at most `2x` plain training
   time at the same seed and step count.
3. **Quality smoke:** on each seed, compare served-evaluator top-1 of the served
   checkpoint against the plain checkpoint. Report the delta and two-seed
   spread. Call served "materially below" only if it is below plain by at least
   `max(24, spread)` on both seeds; likewise for plain. Otherwise the quality
   reading is inconclusive.

The instrument and operational doors decide whether the wiring is ready for a
longer run. The quality smoke never changes a mechanism status.

## Measurements

For every run:

- train-loss trace at steps `0, 64, 128, 192, 256`;
- final checkpoint SHA-256;
- step-256 `--inspect` with `--served-forward 1`, including nearest/writer top-1,
  admitted rows and query spread;
- `--decode-speed` with `--served-forward 1`, reported as decode tokens/s;
- `fbs-snapshot --round-as-exported`: the set of saved files whose bytes change.

The served-forward override is for evaluation only and must not change the
training objective. Before inspection, let `fbs-snapshot` create each separate
evaluation run:

```text
fbs-snapshot --round-as-exported RUN/models/step-000256 EVAL/models/step-000256
```

`EVAL` must not exist before the command. `fbs-snapshot` creates the run folder
and copies the source `config.json` itself; the copied config preserves the
source arm, seed, architecture, reads, rounding flags and arm setting. All later
scoring flags are identical, and the source and copy file hashes are recorded.
This is required for the plain arm, whose unrounded files
`evaluate_dev_panel_d11` correctly refuses. The served arm receives the same
copy step to keep the evaluation path identical.

## Steps

1. Build the exact source with `--features cuda`, verify the binary hashes, and
   run the CUDA library tests.
2. Run the four training jobs in order: plain/s1, served/s1, plain/s2,
   served/s2. After each, fail fast on a non-finite loss or missing checkpoint.
3. Make one rounded evaluation copy per final checkpoint and verify its file
   hashes.
4. Run `--inspect --served-forward 1` and `--decode-speed
   --served-forward 1` on each copy.
5. Seal logs, progress files, configs, source hash, binary hashes, data hashes,
   copy hashes, reports and a `SHA256SUMS`; upload and read back from the store.

## Cost

One RTX 5090 for at most one GPU-hour, ceiling `$1.50`, one pod, inside the
project-wide 4-pod/$8-per-hour caps. Build and tests are projected at 20
minutes; four 256-step runs at 10 minutes; all evaluation at 20 minutes.

The allocation command is:

```text
scripts/pod/uor-pod up --lab deepseek --session served-comparison-1 \
  --count 1 --gpu 5090 --hours 1 \
  --purpose "matched served-surrogate training comparison" \
  --card https://github.com/UOR-Foundation/uor-r4/pull/3567
```

The lease is renewed at least every 30 minutes. It is released and the pod is
destroyed only after `/workspace` results are archived, uploaded, read back and
checked against `SHA256SUMS`.

## Run result

Claude ran the four registered jobs on seed 1's pod at head
`36ca53c81195969f9487747163cc13c50d145efa`, alone on the GPU. The sealed
handoff is `results/claude/one-model-end-to-end-copy-key-2026-11-02/s1/s1.tar`,
SHA-256 `738457b26af18e9176bd0e3ece76f0502d4505cdf0a3f249381fc0ccf574ad1d`.
The local review could not download that private-store object because this
workstation has no Hugging Face credential; the decision uses the sealed run
summary Claude posted on #3407.

| Seed | Mode | Train seconds | Top-1 / 14,287 | Nearest hits | Openings / 1,226 |
|---:|---|---:|---:|---:|---:|
| 1 | plain | 51 | 36 | 225 | 22 |
| 1 | served | 355 | 11 | 23 | 1 |
| 2 | plain | 52 | 61 | 229 | 15 |
| 2 | served | 355 | 6 | 20 | 1 |

- **Instrument PASS:** all four runs completed with finite losses and loaded
  through the served evaluator. Both served checkpoints were byte-identical
  after `--round-as-exported`; four of five files changed for each plain
  checkpoint.
- **Operational FAIL:** served/plain training time was 355/51 = 6.96× and
  355/52 = 6.83×, above the registered 2× limit.
- **Quality inconclusive:** served minus plain top-1 was −25 at seed 1 and −55
  at seed 2. The two-seed spread is 30, so the required band is max(24, 30) =
  30. Seed 2 is materially below but seed 1 is 5 rows inside the band; the
  sheet requires both seeds before calling a material difference.
- **Decode:** with `--served-forward 1`, 49.2–49.4, 40.8–41.1 and 28.6–28.8
  tok/s at contexts 16, 64 and 192. This is decoding, not training throughput.
- **Cause not yet measured:** the 49-pass `activation_quantize16` scale loop is
  a likely cost source, but no profile isolates it. Do not report it as proven.

**Instrument fault found by the run:** the first evaluation wrapper created
`EVAL/config.json` before calling `fbs-snapshot`, so the command correctly
refused all four outputs. The wrapper nevertheless wrote `DONE`. The rerun used
the corrected command above under a fresh root. A future wrapper writes `DONE`
only after every round command's exit status has been checked.

## Minimum test power

Two seeds are the minimum for a mechanism comparison. The short run has 256
steps, not model-fit power; it can establish finite operation, export identity
and a smoke-level quality comparison only. The quality band is
`max(24 rows, two-seed spread)` and cannot establish a training-law win.

## Stop rule

- Stop on any non-finite loss, failed checkpoint load, missing data pin, or
  output hash mismatch.
- Stop if the job would exceed one GPU-hour or $1.50.
- Do not open a fresh panel; DEV is diagnostic here.
- If the served arm fails the instrument door, preserve the run and classify the
  failure as instrument or mechanism before any retry.

## Verdict

TEST FITNESS: FIT - fresh non-author review of sheet SHA-256
`bdf3f781acad77dd0cb55424c72e66d34fb1a4ecc48c10e1409ffd70705538c8`;
PRE-RUN: PASS.

The review confirmed executable rounded evaluation copies for both training
modes, explicit TRAIN/DEV/mask/tokenizer pins, no unsupported top-8 claim, a
two-mode × two-seed scope, finite-loss and budget stop rules, and the
engineering-only claim boundary.

**Applied result:** instrument PASS; operational FAIL; quality INCONCLUSIVE.
The served surrogate is wired and export-consistent, but its 6.8–7.0× training
cost fails the readiness gate. The next unit is a focused quantizer-cost
repair and matched rerun, not a longer language run.
