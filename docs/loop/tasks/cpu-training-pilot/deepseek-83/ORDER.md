# ORDER: cpu-training-pilot, deepseek-83

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `de9cfc84a`.

## Goal

Repair deepseek-82 after exact-head review found that the seed-2 arms used
trainer seed 2 even though the FIT sheet fixed trainer seed 1 for both artifact
seeds.

## Finding

The original seed-2 receipts and later REPORT text silently used artifact seed
as trainer seed. The review caught the preregistration deviation. Seed 1
already fails the candidate admission and ranking bar, so correcting seed 2
cannot create a positive; it is required to make the reported replication
faithful to the registered protocol.

## Repair

- Preserve the superseded trainer-seed-2 result in Git history.
- Rerun seed-2 baseline, K1-step-1 and K4-step-1 with trainer seed 1 into
  fresh output directories.
- Replace only the bound seed-2 receipts and regenerate comparison values,
  `SHA256SUMS`, REPORT, notebook, program and roadmap readings.
- State the deviation and correction in the REPORT rather than presenting the
  superseded run as registered evidence.

## Scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-82/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-83/**`
- `docs/research/notebook/cpu-training.md`
- `docs/research/cpu-training-program.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. All six bound arms record trainer seed 1 and the registered step and
   coordinate settings.
2. The complete repaired evidence parses and matches its checksums.
3. Exact export, expected exit accounting, K4 telemetry and the bounded
   negative are accurate on both artifact seeds.
4. The deviation and its correction are recorded in the REPORT.
5. A fresh exact-head review approves the repaired PR.

## Stop if

An artifact, sample, export, telemetry or admission instrument fails; classify
instrument against mechanism before retrying. Do not tune the metric or
configuration after observing either run.
