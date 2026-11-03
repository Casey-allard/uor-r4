# REPORT: worker-budget evidence repair, deepseek-101

> **Superseded by [deepseek-102](../deepseek-102/REPORT.md).** A second fresh
> review accepted the timing implementation and test but found two retained
> record errors: the comparison command included a worker-specific audit path,
> and this report's audit-time range covered only forward-order runs.

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base:
`6729e5d507bd39ef400c6515bc29b75f97e682d7`.

## What did not work, first

- **The first exact-head review returned CHANGES on four valid findings.** The
  audit timer included atomic publication and reload, trainer setup and gate
  construction were unattributed, the field test was too weak, and the report's
  size and semantic-checksum recipe were stale.
- **The first committed evidence failed the exact-head whitespace check.** The
  gate table and timing logs retained trailing spaces, and the package-test log
  ended with a blank line. The empty diff-check receipt contradicted that state.
- **The release build hit the known cross-worktree `uor-r4-integer` artifact
  collision twice during repair.** Immediate retries passed. The failure and
  retry history remain in the retained evidence.
- **The one-worker runs still take about one minute each.** That is the expected
  serial cost of two 1,024-row exact panels, not a remaining parallelism defect.
- **The worker choice remains workload-specific.** The public panel workload
  improves through 32 hardware threads, while synthetic counterfactual
  candidate search remains faster at 16 physical workers.

## What changed

The corpus driver now prepares and validates the served-forward audit separately
from atomic artifact/receipt publication and reload. The report has explicit,
non-overlapping setup, baseline-panel, trainer-setup, training, final-panel,
gate, checkpoint/reload, audit, publication/reload, post-publish verification
and unattributed intervals. The unattributed interval makes accounting closure
visible instead of hiding small gaps inside another stage.

The driver test now executes under a dedicated three-worker Rayon pool, asserts
that exact reported count, checks the complete stage-key set, and verifies the
stage sum does not exceed wall time.

All eight worker-budget reports and both scheduling comparisons were regenerated
from repaired driver SHA-256
`51d6c1b4c543e24bd5bb1b0fa2b769efb1554521891935e4ddba309a17903ffd`.
The retained text logs are whitespace-normalized for the repository gate, and
the state-comparison recipe is now written beside the reports.

The exact PR diff is 49 files: 5,317 insertions and 17 deletions, with generated
and retained evidence accounting for most of the lines.

## Measurements

Host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.
Each single-artifact run used public dual-c131 seed 1, trainer seed 1, 16 update
attempts, evaluation stride 2, a 1,024-row panel cap and checkpoint interval 16.

| Workers | Forward wall | Reverse wall | Speedup vs one (forward / reverse) |
|---:|---:|---:|---:|
| 1 | 62.045 s | 62.248 s | 1.000x / 1.000x |
| 8 | 8.483 s | 8.475 s | 7.314x / 7.345x |
| 16 | 4.640 s | 4.630 s | 13.372x / 13.446x |
| 32 | 3.534 s | 3.522 s | 17.557x / 17.676x |

The two runs average 17.62x at 32 workers. Across all eight single-artifact
runs, the training stage stayed at 283-288 ms, audit computation at
34.7-35.6 ms, and publication/reload at
7.0-16.6 ms. The two exact panels dominate wall time.

Every run produced final artifact
`6b35fed04be8a98168b7a7a0a22030300de5383b7576c0c06a812f28e7bb38c8`,
final checkpoint
`b3a9c501c42d94f88091f4360a9b0a67dc8d294ebf5bdbb0d58b98de20e6ce24`,
panel `278 / 384 / 396`, served-forward decision hash
`2ac6b0971afe222860453846f286385428bb870e802fccf66a6097965ea1b897`,
and zero query, token or kept-cell mismatches.

Two distinct public artifacts ran as two sequential 32-worker jobs and two
concurrent 16-worker jobs. Sequential took 7.06 s; concurrent took 6.60 s, a
6.5% wall reduction. Each per-seed output matched its sequential counterpart.

## Checks run at the head

- `CARGO_TARGET_DIR=... cargo test --release --locked -p uor-r4-cpu-train
  --lib --bins`: passed on immediate retry after the known collision; 159
  library tests passed with 4 release benchmarks ignored, and all binary suites
  passed.
- `CARGO_TARGET_DIR=... cargo clippy --locked -p uor-r4-cpu-train
  --all-targets --no-deps -- -D warnings`: passed after an immediate retry.
- `cargo fmt --package uor-r4-cpu-train -- --check`: passed.
- `git diff --check origin/main...HEAD`: passed after evidence normalization.
- `uor-harness loop gates --full`: 11 of 11 gates green.
- Evidence `SHA256SUMS`: passed.

## Skills used

No available Codex skill applied.

## Still open

- A training configuration that clears a registered capability or fresh-reply
  bar.
- Unseen-key transfer, trained float shadow and credentialed complete-reply and
  fresh-phrasing evidence under #3506.
- An automatic independent-job scheduler. This unit verifies the measured
  16+16 pattern but does not add launcher automation.
- CPU training remains D84 development research and does not replace GPU
  qualification.

## Plain summary

The initial timing report put artifact writing inside the audit interval and
left small setup gaps unexplained. This repair gives each phase its own clock,
including publication and an explicit remainder, and reruns every measurement
from the corrected binary. It also tests the actual worker count and the
accounting bound. On this 32-thread workstation, 32 workers make a
panel-dominated corpus run 17.62 times faster than one worker, while producing
the same trained files, scores and audit. Two independent jobs can instead run
as two 16-worker lanes and finish about 6.5% sooner than two sequential
32-worker jobs.
