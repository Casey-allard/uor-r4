# ORDER: cpu-training-pilot, deepseek-114 (geometric termination point gate)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `ed5f0d6b2`.

## Goal

Test a geometric termination-point gate. Derive Q12 query points from
teacher-forced training rows that predict BOS, remove training points that
collide with nonterminal queries, and derive a deterministic L1 radius from
the remaining training points and the nearest nonterminal training query.

At served decode, a generated query inside the radius names BOS directly.
Compare the trained gate with no gate and an equal-size random nonterminal-point
control on both public seeds.

This is a new mechanism hypothesis. It does not repeat rank margin, memory
weighting, direct target arbitration, or point voting.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-termination-gate-*`
- `docs/loop/tasks/cpu-training-pilot/deepseek-114/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Registered configuration

- seed 1 and seed 2 public artifacts;
- fixed deepseek-109 16-key / 102-training / 114-evaluation selection;
- 32-update replay to reproduce pinned trained artifacts;
- no gate, trained termination gate and equal-size random nonterminal gate;
- 16 generated tokens;
- 32 Rayon workers;
- seed 1 first, seed 2 second, under the 30-minute stop.

## Done when

1. The gate uses only training-derived terminal query points and a radius
   derived without evaluation data.
2. Finite tests cover endpoint derivation, collision removal, radius selection,
   random control construction, exact BOS override and stop behavior.
3. FIT fixes hashes, arms, bar, controls, cost and stop before public replay.
4. Both seeds reproduce trained artifact hashes and the no-gate reply digests
   from deepseek-110.
5. The REPORT compares BOS-complete, premature-stop, exact-reference and
   reachable exact-reference results for trained, random and no-gate arms.
6. The candidate advances only if it strictly beats no-gate and random on
   BOS-complete replies on both seeds without lowering exact-reference counts.
7. Focused/full CPU tests, strict Clippy, formatting, evidence, loop gates,
   exact-head review, protected merge and fresh-main verification pass.

## Stop

Stop on training/evaluation leakage, gate inputs from evaluation rows,
artifact-hash or no-gate digest drift, missing parity, nondeterministic
gate/radius, counter overflow or post-observation tuning.
