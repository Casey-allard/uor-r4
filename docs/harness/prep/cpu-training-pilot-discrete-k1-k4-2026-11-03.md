# CPU discrete proposal K1/K4 corpus comparison

**Status:** FIT

TEST FITNESS: FIT

## Question

Does committing exact joint BVG-K blocks of four table coordinates improve
held-out ranking beyond the existing one-coordinate discrete proposal loop,
with both arms seeing the same public corpus batches and initialization?

The underlying K-block, exact joint-loss and atomic apply components exist.
The public four-table corpus driver previously hardcoded K1 and stopped before
writing a receipt when admission fell. This comparison adds the bounded K4
driver and tests the actual corpus behavior.

## Base artifacts

Public `caseyallard/uor-r4-m1-discrete`, four-table Stage-T FrozenZero exports:

- seed 1: `t16-frozen-zero-seed1.lut`, SHA-256
  `fc31bcc7a88250f3758d07a8b3d12480a5332acb5437b0c0f0e5c71ae88939ca`;
- seed 2: `t16-frozen-zero-seed2.lut`, SHA-256
  `a75ea7f5907275fd4c1e853ee76135a7a2615ba8302aaadf362bfa7eb96ae12a`.

These are four-table `UORLUT01` exports. The 19-table M1 trunk is outside this
CPU driver.

## Data split

The public DEV UORT token and response-mask inputs and public mask manifest:

- tokens SHA-256
  `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
- mask SHA-256
  `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`;
- manifest SHA-256
  `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab`.

The deterministic split remains 200 training documents / 7,080 targets and
198 evaluation documents capped to 1,024 rows at stride 2. The manifest says
`split=train`; base-model exposure is unverified.

## Arms

Every run uses trainer seed equal to its artifact seed, batch rows 64,
evaluation stride 2, a 1,024-row cap and checkpoints every 64 completed
blocks.

| Arm | Steps | Coordinates per block | Admission flag |
|---|---:|---:|---|
| baseline | 0 | 1 | required |
| control-k1 | 256 | 1 | reported, not required |
| pivot-k4 | 256 | 4 | required |

The K1 control omits `--require-admission` so an admission outcome cannot stop
the active K4 arm. Baseline and K4 exit 2 after writing their receipt if final
admission falls.

## Command template

```sh
cpu-discrete-corpus-driver ARTIFACT TOKENS MASK MANIFEST \
  SEED STEPS 2 1024 64 RUN_DIR \
  --coordinates-per-step COORDINATES [--require-admission]
```

## Bar

- **Instrument:** each arm writes a finite report; 1,024 evaluation rows are
  read; source and final CPU/serving export parity are exact; final artifacts
  reload; checkpoint names and binding hashes agree. Checkpoint continuation is
  covered by focused tests, not by an additional arm in this six-run
  comparison.
- **Expected exits:** seed baseline exits 0; K1 exits 0; K4 exits 0 with
  `panel_gate=accept`, or exits 2 only after writing a complete report with
  `panel_gate=reject_admission`.
- **Update activity:** each trained arm accepts at least one block and reports
  a nonzero accepted-block histogram.
- **Candidate breadth:** K4 must commit at least one accepted block with more
  than one coordinate. `committed_moves > accepted_updates` and the
  move-count histogram must demonstrate the same blocks.
- **Admission:** K4's final target-admitted count is at least baseline.
- **Ranking:** for each seed, `p = baseline_top1 / 1024` and
  `F = ceil(3 * sqrt(1024 * p * (1-p)))`. K4 must gain at least `F` top-1 rows
  over baseline, strictly exceed K1 top-1, and not fall in top-8.
- **Positive:** every row passes on both engine seeds.
- **Negative:** anything else closes this bounded K1/K4 corpus configuration.

## Measurements

Each receipt reports accepted/refused blocks, accepted block-size and
committed-move histograms, cumulative and invocation moves, target visits,
context-token throughput, panel top-1/top-8/admission/margin/rank, source and
final artifact hashes, checkpoint hashes and exact export parity.

Record source commit, release binary hash, workstation identity and every
input hash.

## Steps

1. Verify the release driver and all five artifact/data hashes.
2. Run baseline, K1 and K4 on seed 1 in that order.
3. Repeat in the same order on seed 2.
4. Preserve every report even when baseline/K4 exits 2.
5. Compare the six receipts and write a result receipt plus `SHA256SUMS`.

## Minimum power, cost and stop

Two public artifact seeds, 1,024 document-disjoint rows per seed and 256
completed blocks per trained arm. This is component development evidence, not
fresh transfer or language capability.

Only the named Ryzen 9 9950X is used: 16 physical / 32 logical CPUs, no GPU,
pod, upload or paid API. Stop before 30 minutes. Projected peak RSS is below
256 MiB and retained output below 100 MiB.

Stop on a non-K4 instrument failure, input hash mismatch, non-finite reading,
export mismatch or failed checkpoint/artifact reload. An admission loss is an
outcome and must remain in the receipt. Do not tune after observing a result.

## Claim boundary

A gain is two-seed internal-development evidence for joint discrete
coordinate blocks only. It is not GPU replacement, useful-chat, fresh-transfer
or complete-reply evidence.

## Verdict

TEST FITNESS: FIT - fresh non-author review of design SHA-256
`22e7228502ff90a83f17077d5ae4d703b7fe354daa25bb520d7e40def974e037`;
`may_compute=true`.

The reviewer confirmed that the comparison is distinct from the exhausted
rank-margin configurations, every artifact/data input is pinned, the K1/K4
schedule and seed semantics are explicit, exact joint scoring/apply and
multi-coordinate telemetry are present, admission negatives remain reportable,
the three-sigma/two-seed ranking bar is correctly specified, and the six-run
machine budget stays inside the registered stop rule.
