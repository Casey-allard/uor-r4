# ORDER: cpu-training-pilot, deepseek-110 (memory-weight complete replies)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `5707380fe`.

## Goal

Replay deepseek-109's exact expanded memory-weight training configuration and
extend the evaluation through complete greedy replies. Compare the frozen,
trained, equal-accepted-move random and memory-off artifacts, including the
trained artifact with memory disabled, on the same 114 evaluation examples.
Also compare direct exact-memory target arbitration against a matched
shuffled-target artifact using the same keys and trained weights.

This unit is not another trainer or a width/step increase. It determines
whether the movement weights that improved teacher-forced top-1 and distance
also change complete reply content or termination before another credit rule
is attempted.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-memory-replies-*`
- `docs/loop/tasks/cpu-training-pilot/deepseek-110/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No float-training, `train-in-space`, or Claude #3629 file is changed.

## Registered configuration

- the fixed deepseek-109 seed-1 and seed-2 artifacts, DEV tokens/mask and
  manifest hashes;
- the exact 16-key / 102-training / 114-evaluation selection;
- 32 attempted Q24 movement-coordinate updates per seed, seed 1;
- 32 Rayon workers;
- frozen, trained, equal-accepted-move random, memory-off and
  trained-memory-off arms;
- direct-target frozen, trained and shuffled-target arbitration arms;
- 16 generated reply tokens, BOS/context/cap stopping through `reply_stop`;
- seed 1 followed by seed 2 under a 30-minute local CPU stop.

The rerun must reproduce deepseek-109's trained artifact hashes exactly before
its complete-reply readings are used.

## Done when

1. A finite reply evaluator is implemented and tested for prompt/reply
   boundaries, BOS/cap/context stops, exact-reference normalization,
   reachability, distinct replies, repeated 4-grams and deterministic digests.
2. A TEST FITNESS sheet fixes the eight arms, 16-token cap, hashes, bar, cost
   and stop before either public seed runs.
3. Both seeds reproduce the registered teacher-forced readings and artifact
   hashes, with zero CPU/serving parity mismatches.
4. Complete replies report exact-reference, BOS-complete, cap/context stops,
   opening top-1, distinct non-empty replies, repeated 4-grams, generated
   tokens and aggregate generated tokens/s for every arm.
5. The REPORT leads with the complete-reply negative or positive and does not
   claim semantic correctness or useful chat from exact-reference matching.
6. Focused and full CPU release tests, strict CPU Clippy, formatting, evidence,
   loop gates, exact-head review, protected merge and fresh-main verification
   pass.

## Candidate bar

On each seed, trained memory carries only if it exceeds frozen, equal-move
random and memory-off on exact-reference complete replies over the 104
cap/context-reachable prompts, and does not reduce BOS completion. For each
paired comparison, with `b` and `c` the two discordant exact-reference counts,
the required margin is `max(2, ceil(2*sqrt(b+c)))`.

Direct target arbitration carries only if `target_trained` beats both the
trained query-movement arm and the shuffled-target artifact on exact-reference
complete replies by the same paired margin on both seeds, without lowering BOS
completion.

The 114 repeated-key prompts are a bounded readiness screen. They do not
support a three-sigma, unseen-phrasing or useful-chat claim.

## Cancel and stop

Stop on artifact-hash drift, changed selection, nondeterministic replies,
missing exact parity, counter overflow or any post-observation change to arms,
cap, keys, steps or controls. Do not extend the same coordinate-descent
configuration in this unit.
