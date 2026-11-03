# CPU table BVG-K pivot comparison

**Status:** READY FOR FIT REVIEW

TEST FITNESS: FIT - M2 #2030

## Question

After three bounded CPU results moved the training margin without moving
held-out ranking, does one exact BVG-K block update across four tied-table
coordinates change more held-out predictions than a matched one-coordinate
update with the same total movement?

The experiment is two public artifact seeds and three arms:

- `baseline`: zero updates;
- `control-k1`: one coordinate by 4 Q12 units;
- `pivot-k4`: four coordinates by 1 Q12 unit each.

The arms have the same maximum coordinate movement (four Q12 units across each
row) but different spatial credit assignment.

This is CPU development research under D84. It is not a GPU qualification, a
language-capability claim, or evidence that CPU training replaces Claude's GPU
main track.

## Base artifacts

- Seed 1:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`.
- Seed 2:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256 `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

Both are public raw FBS artifacts, vocabulary 4,096, context 192, native
writer, no Hamming or turn memory.

## Data split

- DEV tokens:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-tokens.u16`,
  SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`.
- Response mask:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-response-mask.u8`,
  SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- Manifest:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-manifest.json`,
  SHA-256 `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab`.

The deterministic document split holds out 1,024 rows across 198 documents and
trains on the remaining 200 documents / 7,080 targets. The source manifest says
`split=train`; base-model exposure is unverified. These are public
internal-development rows, not sealed fresh phrasing.

## Arms

Every arm uses trainer seed 1, 4,096 update attempts, checkpoints every 1,024,
evaluation stride 2 and the 1,024-row cap. Each arm starts from a fresh load of
its pinned artifact.

| Arm | Margin Q12 | Step Q12 | Coordinates per step |
|---|---:|---:|---:|
| baseline | 1 | 1 | 1 |
| control-k1 | 1 | 4 | 1 |
| pivot-k4 | 1 | 1 | 4 |

Command template:

```sh
cargo run --release --locked -p uor-r4-cpu-train --bin cpu-corpus-driver -- \
  ARTIFACT DEV_TOKENS DEV_MASK DEV_MANIFEST \
  1 STEPS 2 1024 1024 RUN_DIR \
  --margin-q12 1 --step-q12 STEP_Q12 --parameter-limit-q12 8192 \
  --coordinates-per-step COORDINATES --require-admission
```

`STEPS` is 0 for baseline and 4,096 for both trained arms.

## Bar

For each artifact seed, compare each trained arm with that seed's own baseline
over the same 1,024 rows.

- **Instrument:** all six processes finish with finite readings and exactly
  1,024 evaluated rows; baseline/final Q12 query, token and kept-cell
  mismatches are zero; the final artifact reloads; the checkpoint binding and
  resume agree.
- **Admission:** the final target-admitted count is at least baseline.
- **Update activity:** each trained arm accepts at least one block and every
  accepted K4 step reports one to four distinct coordinates. The driver records
  the accepted-coordinate histogram and fails if an accepted block violates
  its configured or coordinate bounds.
- **Pivot ranking:** top-1 is the pre-fixed primary metric. For each seed let
  `p = baseline_top1 / 1024` and
  `F = ceil(3 * sqrt(1024 * p * (1-p)))`. K4 must:
  - gain at least its seed's `F` top-1 rows over baseline;
  - not fall in top-8 over baseline; and
  - strictly exceed K1's top-1 on both seeds.
  No post-result switch to top-8 is allowed.
- **Replicated positive:** K4 satisfies the pivot ranking rule on both seeds.
  Anything else is a pivot negative: no further one-coordinate or table-only
  ranking tuning on this line without new causal evidence.

## Measurements

For every run:

- accepted/refused updates, accepted-coordinate histogram and final epoch;
- top-1, top-8, target admission, margin-loss sum and target-rank sum;
- step-only and wall throughput;
- final artifact and checkpoint SHA-256;
- exact CPU/serving query, admitted-cell, top-8 and token parity;
- process peak RSS and machine identity.

## Steps

1. Build release `cpu-corpus-driver`, record source and binary hashes and run
   the focused library, driver and checkpoint tests.
2. Verify every artifact and corpus hash above.
3. Run baseline, K1 and K4 for seed 1 in that order.
4. Repeat in the same order for seed 2.
5. Compare raw panel readings under the bar, write one JSON receipt and
   `SHA256SUMS`, and preserve every failed arm.

## Minimum test power

Two published model seeds, 1,024 fixed document-disjoint rows and 4,096 update
attempts per trained arm. This is enough for a bounded development pivot, not
fresh-phrasing or language evidence.

## Cost and stop rule

The Ryzen 9 9950X host is the only compute. No GPU, pod, rental, upload or paid
API is used. At registration the host had 637,416,054,784 bytes free on the
project filesystem and 7,261,335,552 bytes available, with 16 physical / 32
logical CPUs. Each source artifact is 2,144,960 bytes. Five checkpoints per
trained run put the six-run retained output below 100 MiB; projected peak RSS
is below 256 MiB from the prior CPU pilot. This leaves the project's 128 MiB
filesystem reserve intact.

Every run passes `--require-admission`, which preserves the complete JSON
receipt and exits 2 after a non-accepting panel gate. Stop on hash mismatch,
non-finite or missing readings, failed artifact reload, lost admission or any
CPU/serving export mismatch. Stop the whole comparison before 30 minutes; do
not shorten a completed arm because its ranking is unimpressive, and do not
tune an arm after seeing results.

## Claim boundary

A positive is a two-seed signal for BVG-K table coordinate assignment on one
reused public internal-development panel. It is not fresh-transfer or
generalization evidence; any later gain claim needs a fresh panel. A negative
closes these K1/K4 configurations and triggers the pivot note; it does not
reject geometric training generally.

## Verdict

TEST FITNESS: FIT - fresh non-author review of design SHA-256
`d4da8245676e5fba64e936ba93b15400d4d7fe513e820f5acda17ae3f53d8f84`;
PRE-RUN: PASS.

The reviewer confirmed K1 byte compatibility, K4 coordinate telemetry, the
fixed 3-sigma primary-metric rule, fail-closed admission, checkpoint coverage,
matched samples and baselines, two-seed bounded power, and the public
internal-development claim boundary. Tests and training were not executed
during review.
