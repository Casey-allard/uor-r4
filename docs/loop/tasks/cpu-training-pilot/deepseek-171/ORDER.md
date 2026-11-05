# ORDER: cpu-training-pilot, deepseek-171 (continuation lookahead)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `03f220688`.

## Goal

Make exact-copy lookahead target-sensitive. For each unique depth-1..4
candidate, clone the exact pre-decision forward and feed the candidate. Use the
learned readout on the resulting state to predict its next token, then prefer
candidates whose model-conditioned prediction agrees with the token that
historically followed that exact candidate occurrence.

Deepseek-169 was control-invalid because target shuffling could not affect a
distance-only score. This unit changes the ranking signal while preserving the
same integer-only forward and exact-copy candidate boundary.

## Fixed construction

1. Fit the existing 2,048-bit modulo-code nearest-example readout on generated
   longest-32 states from train panel 2711, with the 90% threshold and
   1,024-row cap.
2. At each decision, retain each unique exact-copy candidate token and its
   source occurrence at depths one through four, choosing the deepest
   occurrence for duplicate tokens.
3. Clone the exact current forward, feed the candidate, and read the learned
   predicted next token and distance from the resulting state.
4. Rank by continuation mismatch, next-state distance, deeper source depth and
   lower token id. If the best distance exceeds the learned threshold, fall
   back to longest-32.
5. Repeat with the deterministic target-shuffled readout.

Controls are no-lookahead learned/shuffled reads, shuffled continuation
lookahead, longest-32, longest-4, `k2>k1`, fixed depth one, random-32, writer
and static sign-Hamming.

## Data and power

- train/fit on panel 2711;
- fresh `cell=dev,dev`, seed 2765, only after FIT and GO;
- two artifact seeds and at least 200 reachable replies each;
- at least one continuation match and candidate arbitration per fresh seed;
- non-vacuous changes versus longest, no-lookahead and shuffled lookahead;
- fresh-process repeats of both seeds.

## Bar

For both artifact seeds on the fresh panel, continuation lookahead must beat
every control on correct body positions by the paired two-sigma rule. It must
change at least one token from longest-32 and no-lookahead learned, preserve
zero premature BOS and ungated mismatches, and repeat byte-identically after
timing removal.

## Stop rule

Stop on source, artifact, tokenizer, panel, readout or report drift; failed fit
or continuation coverage; fewer than 200 reachable replies; vacuous controls;
nondeterminism; or any need to change serving, CUDA, float training,
termination or Claude-owned source. Once seed 2765 is read, do not retune on it.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-copy-continuation-2026-11-05.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-171/**`
- `docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`

## Done when

1. Continuation-sensitive lookahead has independent finite tests and
   deterministic fallback.
2. FIT and GO precede seed 2765.
3. Both artifacts run with deterministic repeats and compact evidence.
4. REPORT classifies the result without claiming complete replies or language.
5. Reviewed merge, fresh-main verification and claim release complete.
