# REPORT: bounded CPU rank-margin step comparison, deepseek-9

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `9c9b712a4`. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Larger rank-margin steps did not produce the registered ranking gain.**
  Seed 1 unit-step training lost one admitted target; step-16 lost one top-8
  row; step-64 gained only 3 top-1 rows against a required 43.
- **Seed 2 rejected both larger steps.** Step-16 left top-1 and top-8 unchanged;
  step-64 exchanged 1 top-8 gain for 3 top-1 losses. That fails both the
  no-trade rule and the ranking bar.
- **A larger step moved the optimization objective but not useful ranking.**
  Step-64 reduced seed 1's margin sum by 122,613 and rank sum by 2,295, while
  top-1 moved 278 to 281. The panel's required 43-row movement did not follow.
- **No GPU or sealed-panel inference is made.** This is a public CPU
  internal-development experiment; complete replies, fresh phrasing and
  language capability remain unmeasured.

## What changed

- `cpu-corpus-driver` accepts positive `--margin-q12`, `--step-q12` and
  `--parameter-limit-q12` overrides while retaining the original defaults when
  they are absent.
- Checkpoint binding now includes all three update-geometry values. Resume
  refuses a checkpoint whose margin, step or parameter limit differs.
- Backward-compatible sidecar defaults preserve resume of pre-change
  checkpoints when they used the original geometry.
- Added a pre-registered two-seed, four-arm comparison (baseline, unit,
  step-16, step-64) over 1,024 fixed document-disjoint rows per seed.

## Measurements

All arms evaluated the same 1,024 rows. Export parity checked 262,144 Q12
coordinates and 8,192 kept-cell entries per arm; every query, admitted-cell,
top-8 and token mismatch count was zero.

| Seed | Step Q12 | Accepted / refused | Top-1 | Top-8 | Admitted | Margin sum | Verdict |
|---:|---:|---:|---:|---:|---:|---:|---|
| 1 | baseline | 0 / 0 | 278 | 384 | 396 | 11,739,473 | control |
| 1 | 1 | 125 / 899 | 278 | 383 | 395 | 11,737,022 | admission loss |
| 1 | 16 | 126 / 898 | 278 | 383 | 396 | 11,706,154 | top-8 loss |
| 1 | 64 | 125 / 899 | 281 | 384 | 396 | 11,616,860 | +3 < +43 bar |
| 2 | baseline | 0 / 0 | 224 | 285 | 292 | 29,717,075 | control |
| 2 | 1 | 53 / 971 | 224 | 285 | 292 | 29,715,877 | unchanged |
| 2 | 16 | 54 / 970 | 224 | 285 | 292 | 29,447,771 | unchanged |
| 2 | 64 | 55 / 969 | 221 | 286 | 292 | 29,296,044 | top-1 loss |

Throughput:

- seed 1 updates: 61.38-61.40 visits/s; 16.286-16.292 ms per update;
- seed 2 updates: 68.64-68.73 visits/s; 14.550-14.568 ms per update;
- active-run wall time: 75.761-78.488 s; baseline wall: 61.208-61.526 s;
- peak RSS: 50,816-52,556 KiB.
- every arm ended at local trainer epoch `0` (1,024 attempts did not exhaust
  one 7,080-target permutation).

Machine: Ryzen 9 9950X, 16 cores / 32 threads, Rust 1.97.1, Linux
`7.0.0-42-generic`. External cost: `$0`. Source `15e52c068`; release driver
SHA-256 `6394604e49f6ca88c315dc85e3934cc6f967e27d0e1ab106f609b800e1d85d3c`.

Final artifact and checkpoint hashes, report hashes and all per-arm readings
are in [rank-step-comparison.json](evidence/rank-step-comparison.json) and its
`SHA256SUMS`. The epoch-instrumented rerun reproduced every prior artifact,
checkpoint and panel reading byte-for-byte.

## Checks run at the head

- Fresh non-author FIT / PRE-RUN PASS on prep sheet SHA-256
  `10944b513fcf51d1c7550be8bd2be1e759a1dc6f0e0093a0f3c1c6eb6dfba3b8`.
- `cargo test -p uor-r4-cpu-train --release --bin cpu-corpus-driver`:
  4 passed.
- Full `uor-r4-cpu-train --release --lib`: 125 passed, 0 failed and 4 release
  benchmarks ignored.
- `cargo clippy -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`:
  passed.
- The inserted source lines have no rustfmt diff. File-wide `rustfmt --check`
  still reports pre-existing import-order differences in this driver.
- Report/pr checks, evidence checksums and exact-head review: recorded in the
  PR review record.

## Skills used

No repository skill was invoked.

## Still open

- The tested 1-, 16- and 64-Q12 steps are configuration negatives; this does
  not reject rank margin or every possible credit assignment.
- A different lawful CPU objective or score-preserving update may be needed to
  move held-out ranking rather than only the local margin.
- GPU training comparison, fresh complete-reply panels and useful local chat
  remain outside this CPU experiment.

## Plain summary

The prior CPU trial moved one integer coordinate by one unit and did not change
ranking. This experiment made the existing coordinate step configurable and
tested 16 and 64 units against matched controls on two public model seeds. The
larger steps changed the training margin and parameter artifact, but they did
not move enough held-out predictions and sometimes lost an admitted target or a
top-8 row. Exact serving export remained intact throughout. The tested
step-size hypothesis is negative, but the CPU training and evaluator path still
works.
