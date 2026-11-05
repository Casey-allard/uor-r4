# REPORT: cpu-training-pilot, deepseek-188

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `d634c77e5`; final implementation:
`771c99ecf`.

## Status

Complete as a two-seed **CONTROL_FAULT**. Raw-conditioned query-port moves
preserved the learned input table and exact serving parity, but the
shuffled-cell arm was behaviorally identical to the trained arm. No model
change is promoted.

## What did not work

The first sealed discovery runs at binary `f3147470…` reported eight accepted
moves on seed 1 and five on seed 2. The trained and shuffled-cell artifacts
were byte-identical on both seeds. Artifact equality alone was too weak to
detect a control with different bytes but identical behavior, so exact review
rejected that repair. The final driver now compares ordered token and kept-cell
decision digests plus all panel and port metrics while excluding artifact
identity.

Fresh verification at binary `7210266b…` reproduced the collision and correctly
returned **CONTROL_FAULT** on both seeds.

The trained arm beat the independent random-gain arm on target-port distance
on both seeds. The invalid shuffled control means this run is not recorded as
either a mechanism REJECT or KEEP.

## What changed

- Added `--baseline-mode raw` to `cpu-query-port-driver`; anchored remains the
  default and keeps its existing arm/output names.
- Raw mode uses the source artifact byte-for-byte and does not reconstruct or
  substitute a token table.
- Strengthened port isolation to restore the original final-gain table and
  require the entire artifact to become byte-identical.
- Split admission, top-1 and top-8 safety fields.
- Added an ordered behavioral signature over token/kept-cell decisions and all
  panel/port metrics, excluding artifact hashes.
- Added fail-closed `CONTROL_FAULT` when shuffled and trained behavior match.

No serving, CUDA, float-training, exporter or Claude-owned source changed.

## Measurements

- FIT head: `ad4340df0`; binary `f3147470…`;
- final control repair: `771c99ecf`; binary `7210266b…`;
- training power: **235** conversations / **2,708** response rows;
- panel power: **58** conversations / **616** response rows;
- seed 1 verification: **71.766 s**, peak RSS **157,000 KiB**;
- seed 2 verification: **52.274 s**, peak RSS **154,856 KiB**;
- accepted/shuffled attempts: **8/8** on seed 1 and **5/5** on seed 2;
- every arm: zero query-coordinate, token and kept-cell parity mismatches.

Verification readings:

| Seed | Arm | Admitted | Top-1 | Top-8 | Port distance |
|---|---|---:|---:|---:|---:|
| 1 | raw | 245 | 181 | 236 | 29,882,444 |
| 1 | trained | 253 | 185 | 241 | 29,613,365 |
| 1 | random | 246 | 182 | 236 | 29,818,562 |
| 1 | shuffled | 253 | 185 | 241 | 29,613,365 |
| 2 | raw | 171 | 136 | 164 | 23,870,298 |
| 2 | trained | 171 | 137 | 164 | 23,786,304 |
| 2 | random | 171 | 138 | 164 | 23,802,494 |
| 2 | shuffled | 171 | 137 | 164 | 23,786,304 |

The trained and shuffled decision digests were identical on both seeds.
Compact evidence:
[raw-query-port-summary.json](evidence/raw-query-port-summary.json).

## Checks run at the head

- full CPU package: **351 library tests passed**, 4 expected ignores, all
  binary tests passed;
- focused query-port driver: **8 tests passed**, including identical behavior,
  changed decisions and changed port metrics;
- scoped strict Clippy:
  `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`;
- FIT, D67 GO, rustfmt, `git diff --check`, D23/D32 ratchets and claim wording;
- both original discovery reports and both final verification reports sealed;
- source artifact identity, non-gain byte identity and exact CPU/serving parity.

## Delivery

PR #3742 was reviewed at exact head
`eddaf1cb479126aede8579223c83b3b8ff1a617d`. All required checks passed and
the protected queue merged it as
`9577aebc9b7dbb0964906efffd13662b7d4984c8`.

Fresh detached `origin/main` passed the full CPU package (**351 library tests
passed**, 4 expected ignores, all binary tests), scoped strict Clippy,
formatting, diff, report/FIT evidence and the `CONTROL_FAULT` evidence check.

## Size

This unit adds one raw mode to the existing CPU query-port driver, focused tests,
the pre-registration and records. No model artifact or corpus is committed.

## Skills used

No system-provided Codex skill applied. Repository pre-registration, D67,
evidence and delivery procedures were used.

## Still open

A future query-port experiment needs a behaviorally distinct cell control. The
one-step cell rotation is closed for this exact raw-conditioned configuration.
The main design implication remains to preserve learned input identity while
adding writer address structure. This run does not yet supply a valid
mechanism comparison for the query-port objective.

## Plain summary

Training the query's two port blocks without touching the input table remained
exportable and exact, but the cell-rotation control collapsed onto the same
behavior as the trained arm on both seeds. The result is a control failure, not
a valid mechanism result. The trained moves beat random gain but cannot be
separated from the behaviorally identical shuffled control, so nothing is
promoted.
