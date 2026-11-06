# Pre-registration: content-continuation boundary guard

Run ID: `cpu-training-pilot-continuation-guard-2026-11-06`

## Status

Complete as **REJECT**. FIT/GO were recorded at `e3f09dd41`. The guard
suppressed 23 boundary decisions but reduced exact replies from 17 to 14 and
completed only 202/223 of 467 reachable replies. Repeats matched.

## Question

Can a confident generated full-sequence continuation prevent premature BOS and
preserve deepseek-198 content without any retrieved-reply endpoint?

## Fixed construction

At every boundary hit, use the same full-history suffix table and strict
majority rule as content selection. If a confident prediction exists and its
depth is greater than local copy depth, suppress BOS and continue with the
normal content choice. Otherwise accept BOS.

Arms: guarded boundary, boundary, suffix-shuffled guard, local longest, full
retrieval, reference-length oracle.

## Data split

TRAIN `tt-2711` with pinned tokens
`140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`,
mask `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`,
manifest `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`.

Fresh panel: `cell=dev,dev`, seed 2786, protocol 2, 128 conversations.

## Instrument gate

Pins match; panel and output absent before GO; suffix table has zero BOS
targets; shuffle preserves counts and changes assignments; all arms share the
panel/artifact; every reachable row completes; repeats match.

## Bar

Guarded boundary must emit all reachable rows with zero premature BOS, retain
the better boundary/local exact and body counts, beat shuffled continuation on
exact replies by two sigma, and change at least one complete reply.

## Outcomes

KEEP if all pass; REJECT if active and a quality/termination bar fails;
INSUFFICIENT_POWER below 200 reachable or 100 guard decisions; CONTROL_FAULT
on drift, vacuous control or nondeterminism.

## Stop rule

Do not retune confidence, depth comparison or controls after seed 2786 opens.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Base and inputs

- base `31974240d`;
- tokenizer
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- artifact seed 1
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- artifact seed 2
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

## Output

`local/deepseek/continuation-guard/2026-11-06`; absent at registration.

## Steps

Implement the generated-state boundary guard and tests, run the full CPU
suite, freeze FIT/GO, generate seed 2786, run both artifacts and repeats, seal
evidence and deliver.

## Minimum test power

Two artifacts, 200 reachable rows, 100 suppressed/accepted boundary decisions,
boundary/shuffled/local/full/oracle controls and deterministic replay.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 continuation-guard sheet.
reviewer: deepseek exact-head self-review (independent delegation unavailable)
note: The guarded endpoint suppresses BOS only when a confident full-history
  successor is deeper than local copy and otherwise accepts BOS. A focused
  test covers guarded completion; full CPU library passes 408 tests with four
  expected ignores, and driver tests pass.
source: `79351f9d9e2708ad8849ab3c4c03b2fb722fd34af88eca62912b3b2579553a75`;
driver: `c1bda7c14751626600e39f0460126e914e805a9c3b0da8732599b4df8c8c30b7`;
release_binary: `381ffa820993d7b8b7d52a3ce2db7dbeeee835a119a30d5926fdb4fa223e0f6a`;

## Pre-run GO

GO at `e3f09dd41`; source, driver and binary hashes match. Seed 2786 may run.
