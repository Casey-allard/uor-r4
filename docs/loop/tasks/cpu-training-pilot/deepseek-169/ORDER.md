# ORDER: cpu-training-pilot, deepseek-169 (copy lookahead)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `97c4c5a4a`.

## Goal

Test whether one-step model lookahead in geometric state space can arbitrate
among exact-copy candidates better than the rejected no-lookahead learned read
and the established depth controls.

## Fixed construction

For each bounded reply decision:

1. Collect the unique exact-copy candidate tokens at depths one through four.
2. For each candidate token, clone the exact `CpuFbsForward`, feed that token,
   and obtain the resulting Q12 writer query.
3. Pack that candidate-conditioned next state as the existing 2,048-bit
   modulo-256 code and read the train-fitted exact nearest-example table.
4. Rank candidates by next-state readout distance, deeper original copy depth
   and lower token id. If no candidate's next state is within the learned
   threshold, fall back to longest-32.
5. Repeat with the deterministic target-shuffled readout.

The readout is fitted independently for each artifact seed from generated
longest-32 states on train panel 2711, using the existing 90% leave-one-out
threshold and a deterministic 1,024-row cap. The fresh panel is seed 2764,
generated only after FIT and GO.

Controls are the no-lookahead learned and shuffled reads from deepseek-167,
lookahead with shuffled targets, longest-32, longest-4, `k2>k1`, fixed depth
one, random-32, writer and static sign-Hamming.

## Data and power

- two artifact seeds;
- at least 200 reference-reachable replies per seed and panel;
- at least one lookahead ranking per fresh seed;
- non-vacuous lookahead-versus-longest, lookahead-versus-no-lookahead,
  shuffled-lookahead and random-32 changes;
- fresh-process repeats of both seeds.

## Bar

For both artifact seeds on the fresh panel, lookahead correct body positions
must beat every control by the paired two-sigma rule. It must change at least
one token from longest-32 and from the no-lookahead learned arm, preserve zero
premature BOS and ungated mismatches, and repeat byte-identically after timing
fields are removed.

Exact and BOS-complete counts remain report-only because stopping is unchanged.

## Stop rule

Stop on source, artifact, tokenizer, panel, readout or report drift; failed
fit coverage; absent lookahead rankings; fewer than 200 reachable replies;
vacuous controls; nondeterminism; or any need to change serving, CUDA, float
training, termination or Claude-owned source.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-copy-lookahead-2026-11-05.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-169/**`
- `docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`

## Done when

1. Lookahead has independent finite tests, candidate-only semantics and
   deterministic rollback.
2. FIT and GO precede seed 2764.
3. Both artifacts run on train and fresh panels with deterministic repeats and
   compact evidence.
4. REPORT classifies the result without claiming complete replies or language.
5. Reviewed merge, fresh-main verification and claim release complete.
