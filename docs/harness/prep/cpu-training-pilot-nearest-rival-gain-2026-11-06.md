# Pre-registration: nearest-rival served-gain training

Run ID: `cpu-training-pilot-nearest-rival-gain-2026-11-06`

## Status

COMPLETE - **CONTROL_FAULT**. A cyclic shuffle changed 2,529/2,538 targets but
the true and shuffled trainers selected identical moves and artifacts. The
registered stop rule ended the run before seed 2 or the repeat. Record:
[deepseek-221](../../loop/tasks/cpu-training-pilot/deepseek-221/REPORT.md).

## Question

Can an eight-rival geometric hinge trained through the exact served final-query
path move held-out top-1 where deepseek-97's target-distance objective did not?

## Fixed construction

Begin from the deepseek-97 memory-preconditioned gain baselines:

- seed 1:
  `3ad89c7b80df52f9e1e4d878c072982e9fb613b8453ded0dd50c3ee905e0ee44`;
- seed 2:
  `070d66364112d4527d3043998f023f435ec73455b9a617aa94695403c62bd089`.

Use that run's fixed 58/56 repeated-key split. For each of eight steps, replay
the current gain through the exact CPU FBS query, select the eight nearest
writer rows other than the target by L1 at Q12, and evaluate all final-gain
coordinates at `+1` and `-1`.

The step objective is:

`sum(max(0, 1 + target_distance - rival_distance))`

over every decision and its eight rivals. Ties choose lower target distance,
then more top-1 decisions, then first coordinate and sign. Rival identities
are fixed within a step and refreshed from the accepted artifact for the next
step. The recurrent trajectory, memory movement, point fallback, Q12 query and
writer table stay exact.

Arms: frozen gain baseline, nearest-rival trained gain, equal-move random gain,
nearest-rival training with a cyclic target-label shuffle, memory-off and
trained-memory-off.

## Data split

The public repeated-key panel from deepseek-97: 58 even-document training
samples and 56 odd-document evaluation samples from eight shared eight-token
keys, on both public FBS artifacts. Input hashes are those in
`docs/harness/prep/cpu-training-pilot-gain-2026-11-03.md`.

## Instrument gate

Frozen memory baselines, source artifacts, corpus, tokenizer and manifests must
match their pinned hashes. The target shuffle must change at least one target.
Every artifact must differ from its baseline only in `final_norm_gain`. Every
accepted step and evaluation arm must have zero CPU/serving query, token and
kept-cell mismatches. The repeat must match byte-for-byte after timing removal.

## Bar

On both artifacts, the true-target arm must:

- accept at least one of eight moves;
- lower held-out multi-rival hinge below frozen, random and shuffled-target;
- improve paired held-out top-1 over frozen by more than
  `3 * sqrt(frozen_only + trained_only)`;
- retain at least the random and shuffled-target top-1;
- pass every instrument gate.

## Outcomes

**KEEP** if every bar passes; **REJECT** if training is active and a quality
bar fails; **CONTROL_FAULT** on input, shuffle, parity, artifact-isolation or
determinism failure.

## Stop rule

Do not retune the rival count, margin, steps or bar after the first scored
artifact. Stop on any prohibited path, drift, stop-budget breach or failed
instrument gate.

## Cost

Local CPU only; at most 30 minutes per process, 2 GiB peak RSS and 64 MiB
retained evidence.

## Base and inputs

- base: `4fde31439`;
- source artifacts:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`
  and
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- tokens:
  `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
- mask:
  `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`;
- manifest:
  `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab`;
- frozen gain baselines as above.

## Output

`local/deepseek/nearest-rival-gain/2026-11-06`; fresh output roots absent at
registration.

## Steps

Build `cpu-nearest-rival-gain-driver`; verify all input and frozen-baseline
hashes; train and evaluate both public seeds; repeat seed 1 in a fresh process;
seal reports, timing, artifacts and source/binary pins.

## Minimum test power

- two artifacts;
- 2,431 held-out decisions per artifact;
- 58 training samples and eight accepted-or-attempted moves per artifact;
- frozen, random, shuffled-target and memory-off controls;
- paired top-1 and multi-rival hinge comparisons;
- exact parity and deterministic replay.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 nearest-rival served-gain training
sheet: docs/harness/prep/cpu-training-pilot-nearest-rival-gain-2026-11-06.md
reviewer: deepseek exact-head self-review (independent delegation unavailable in this run)
note: Implementation 2ba8b56b6 adds an exact eight-rival final-gain trainer and
  driver over deepseek-97's frozen memory baselines. Rival identities refresh
  once per step, all 512 gain candidates use the exact served query and fixed
  step rivals, and ties are deterministic by hinge, target distance, top-1,
  coordinate and sign. The cyclic target-label shuffle preserves the target
  multiset and refuses a vacuous shift. Three focused tests cover fixed-vs-
  refreshed rival agreement, target exclusion/uniqueness, shuffle matching,
  lexicographic improvement and final-gain-only mutation. The full CPU library
  passes 402 tests with four expected ignores; scoped Clippy passes with the
  pre-existing type-complexity lint exempted; formatting and diff checks pass.
  Source, baseline, corpus and output-root absence were verified.
source:
  `db819022a1f5a4ad6dd1db5fd61a854352cf0d902683d20aaf0b453e97665e62`;
driver:
  `2087f747509282d5d62b6239ed6ce68ed4c1935dcf73ebe8d83c6997eb61eb01`;
release_binary:
  `542a2d9980dec9b57eaa2e107427df0919c166d70ffebaf271335ad8b3e80e3b`;
date: 2026-11-06

## Pre-run GO

GO - exact-head self-review under the current disabled-delegation policy.
Implementation `2ba8b56b6`, source
`db819022a1f5a4ad6dd1db5fd61a854352cf0d902683d20aaf0b453e97665e62`,
driver
`2087f747509282d5d62b6239ed6ce68ed4c1935dcf73ebe8d83c6997eb61eb01`
and release binary
`542a2d9980dec9b57eaa2e107427df0919c166d70ffebaf271335ad8b3e80e3b`
match. This was not an independent review. The two public seeds and one
fresh-process repeat may now run under the fixed construction and stop rule.
