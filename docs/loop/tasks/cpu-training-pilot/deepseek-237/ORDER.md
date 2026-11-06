# ORDER: cpu-training-pilot, deepseek-237 (decoupled readout refit)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `1f2bd76ce`.

## Goal

On the public read-free raw FBS artifacts, test whether the nearest-token
output table can be refit from exact TRAIN queries while the input embedding,
SpiralCore/FBS trunk and geometric writer-cell admission path remain fixed.
These artifacts declare `read=dot` and contain no Hamming, turn-memory or point
tables; this item does not test a Hamming path.

## Fixed construction

Teacher-force each public FBS artifact over TRAIN `tt-2711` and capture the Q12
query immediately before every masked reply target. Leave the input table, FBS
trunk and writer-cell admission path unchanged. Refit only output-table rows
with at least eight observations:

- coordinate-wise integer median (primary L1 candidate);
- integer mean centroid;
- deterministic matched label shuffle preserving target counts and the global
  query multiset;
- shared-table refit diagnostic, which changes the input path as well.

Keep every under-observed row unchanged.
The driver enforces seed 2789, the eight-observation floor, tokenizer
`a9fb13f7…`, BOS 0 and exactly one causal observation per masked target.
The pre-run command verifies every source and input hash and reruns both
TRAIN-only preflights before the fresh panel is created.

## Arms

1. original shared table;
2. decoupled identical table;
3. output median refit;
4. output mean-centroid refit;
5. shuffled-target matched control;
6. shared-table median refit diagnostic.

## Data and power

Fitting uses TRAIN `tt-2711` only. Fresh evaluation is `cell=dev,dev`, seed
2789, protocol 2, 128 conversations, both public artifacts, generated only
after FIT and GO. Need at least 100 refitted rows and 1,000 evaluation targets.

## Bar

Decoupled-identical must reproduce original decisions exactly. On both
artifacts, median output refit must strictly improve top-1 over original and
shuffled control by the paired rule
`candidate_only >= max(2, ceil(2*sqrt(candidate_only + control_only)))`, and
top-8 must not fall. Evidence and repeated decisions must be deterministic.
Only the four-report aggregator may issue the final KEEP, REJECT,
INSUFFICIENT_POWER or CONTROL_FAULT result.

## Stop rule

Stop on drift, unsupported artifact shape, fewer than 100 refitted rows, fewer
than 1,000 evaluation targets, vacuous shuffle, nondeterminism, any float or
dense path, or a failed bar. Do not retune the threshold, rounding or shuffle
after seed 2789 opens.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.
The source, hardware and prospective resource record are in
[`resource-ledger.json`](../../../../labs/cpu-training-pilot-readout-refit-2026-11-06/resource-ledger.json).
