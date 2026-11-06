# ORDER: cpu-training-pilot, deepseek-204 (prompt-set reply retrieval)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `77f189f58`.

## Goal

Test a new completion path after continuation closure failed: retrieve one
complete TRAIN reply by exact discrete similarity between the evaluation
prompt's token set and each TRAIN prompt's token set, then emit that reply with
BOS. This removes tokenwise termination from the completion arm and adds prompt
context that full-history suffix lookup did not expose.

Primary similarity is exact unigram-set Jaccard, ordered by integer
cross-multiplication. Equal scores choose the lowest TRAIN ordinal. Neither
matrix multiplication, probabilistic normalization nor float serving is used.

This is D84 CPU development research. It is exact-memory retrieval, not general
reasoning, useful chat or GPU qualification.

The earlier continuation units retained tokenwise generation and changed only
the stop source, key or closure mode. Existing trace signals did not robustly
separate endpoints. This unit instead retrieves one complete response. Closed
seed 2776 was exploratory only: its ordinal-tie arm reached 52/449 exact with
zero premature stops, versus 17 exact and 71/64 premature for deepseek-202.

## Arms

1. `unigram_jaccard_ordinal_tie`: treatment;
2. `bigram_jaccard_geometry_tie`;
3. `unigram_jaccard_shuffled_replies`: deterministic permutation of complete
   TRAIN replies;
4. `query_geometry_nearest`: exact Q12 writer-query L1 nearest prompt;
5. `fixed_ordinal`: always return the first TRAIN reply;
6. `reference_membership_oracle`: diagnostic upper bound that returns the
   reference only when that complete reply occurred anywhere in TRAIN.

The Q12 tie arm remains a control; seed 2776 showed no gain from geometry.

## Data and power

- TRAIN from panel `tt-2711` only;
- fresh unopened `cell=dev,dev`, seed 2777, protocol 2, 128 conversations;
- both public dual-c131 raw artifact seeds;
- 64-token cap and at least 200 reference-reachable replies per artifact;
- at least 200 panel prompts without an exact TRAIN prompt match;
- at least 100 TRAIN reply associations changed by the deterministic shuffle;
- fresh-process repeats of both artifact seeds.

## Bar

For both artifact seeds, `unigram_jaccard_ordinal_tie` must:

- produce at least `max(25, ceil(reference_membership_exact / 8))` exact
  replies;
- beat bigram Jaccard, shuffled replies and fixed ordinal on exact replies by
  the paired two-sigma rule;
- preserve at least as many correct body positions as bigram Jaccard and
  writer-query geometry;
- emit a complete BOS-terminated result for every reference-reachable row;
- report zero premature BOS;
- show that the Q12 geometry-tie control is not superior in exact replies or
  body positions;
- repeat exactly after timing fields are removed.

The shuffled control must be behaviorally non-vacuous. A changed complete
reply on fewer than 100 rows is `CONTROL_FAULT`. Fewer than 200 reachable rows
is `POWER_LIMITED`.

## Cost

Local CPU only on the Ryzen 9 9950X; no GPU, pod, paid API or upload. The
30-minute run cap uses 32 Rayon workers, 2 GiB peak RSS and 32 MiB retained
evidence. Builds, panel, runs, evidence and delivery have a 36-minute bound.

## Done when

1. The exact token-set memory, controls, CLI and focused tests are implemented.
2. FIT and GO precede fresh seed 2777 and both artifact runs with controls and
   repeats.
3. REPORT, evidence and program records are complete, followed by protected
   merge, fresh-main verification and claim release.

## Stop rule

Stop on source, artifact, panel, table, retrieval, parity, determinism or
output-root drift; fewer than 200 reachable replies; fewer than 100 changed
shuffled associations; a vacuous control; any need to edit serving, CUDA,
float-training, exporter or Claude-owned source; wall time above 30 minutes;
peak RSS above 2 GiB; or retained evidence above 32 MiB.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-204/ORDER.md
