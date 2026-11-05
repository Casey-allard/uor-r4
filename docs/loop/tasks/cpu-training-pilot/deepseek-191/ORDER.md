# ORDER: cpu-training-pilot, deepseek-191 (exact reply-boundary memory)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `86d654170`.

## Goal

Test whether termination can be carried by exact memory of observed reply
boundaries rather than another learned stop classifier. Build a reply-only
suffix table from the TRAIN panel and let a suffix that was seen only at reply
end emit decoder BOS while generated tokens continue to use the established
`Longest32To1` exact-copy selector and writer fallback.

This is D84 CPU development research. It is exact-memory recall evidence, not
new-wording generalization, useful-chat qualification or GPU qualification.

## Why this is next

- `deepseek-153` showed that longest-first exact copy carries substantially
  more correct body tokens than its controls but cannot terminate because its
  copy domain has no BOS.
- `deepseek-155` showed an oracle stop would recover a bounded complete-reply
  signal, but the actual decoder exposed none.
- `deepseek-157` and `deepseek-159` trained stop classifiers over generated
  query states; both failed to transfer to fresh generated bodies.
- `deepseek-110` derived reply-memory keys only for nonterminal next-token
  transitions. It never mapped observed reply endings to the decoder BOS.
- This unit tests termination as the next symbol of an exact observed reply
  continuation, with no new model-state classifier.

## Fixed construction

Add an isolated reply-boundary module and evaluator in
`uor-r4-cpu-train`.

For every TRAIN reply:

1. remove exactly one trailing corpus EOS/BOS marker;
2. for depth `d = 4..=32`, count the final `d`-token suffix as terminal;
3. count every internal `d`-token suffix as nonterminal.

At generation:

1. use `Longest32To1` exact copy for body tokens, with writer fallback;
2. inspect only the generated reply suffix, never the prompt;
3. from longest to shortest depth, use the first known suffix;
4. emit decoder BOS only when that suffix has terminal observations and no
   continuation observation;
5. continue when the longest known suffix has any continuation observation;
6. leave the existing no-boundary API and behavior unchanged.

## Arms

Per public artifact seed:

- `longest32_no_boundary`;
- `longest32_true_boundary`;
- `longest32_shuffled_boundary`;
- reference-length oracle for ceiling reporting only.

The shuffled control deterministically moves terminal counts across suffix
keys separately at each depth and leaves every nonterminal count fixed. It
preserves the number of keys and terminal observations at each depth.

## Data and power

- train boundary table from the fixed `tt-2711` panel;
- fresh unopened `dev,dev` panel seed 2723, generated only after FIT and GO;
- public dual-c131 artifact seeds 1 and 2;
- 64-token generation cap;
- at least 200 reachable replies per artifact and arm;
- fresh-process repeat of both artifact seeds.

## Bar

If the reference-length oracle ceiling is below 10 exact complete replies,
return `INSUFFICIENT_POWER`.

Otherwise, on both artifact seeds, the true boundary must:

- produce at least `max(5, ceil(oracle / 2))` exact complete replies;
- beat no-boundary and shuffled-boundary on exact complete replies by the
  existing paired margin `max(2, ceil(2*sqrt(b+c)))`;
- keep premature BOS at or below 4 per panel;
- keep correct body positions at or above the no-boundary arm;
- change at least one stopping decision under the shuffled control;
- reproduce every reading and digest in a fresh process.

## Done when

1. The isolated module, evaluator and focused tests are implemented.
2. A FIT-reviewed pre-registration and fresh D67 GO are recorded.
3. The fresh panel and both artifact seeds run with all controls.
4. REPORT, compact evidence, notebook, item/current state and research program
   record the result.
5. Exact-head review, protected merge, fresh-main verification and claim
   release complete.

## Stop rule

Stop on source, artifact, panel, boundary-table, parity, determinism or
output-root drift; fewer than 200 reachable replies; a vacuous shuffled stop
control; any need to edit serving, CUDA, float-training, exporter or
Claude-owned source; wall time above 30 minutes; or peak RSS above 2 GiB.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-191/ORDER.md
