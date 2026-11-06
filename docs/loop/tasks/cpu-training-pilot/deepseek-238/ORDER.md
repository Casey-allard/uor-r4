# ORDER: cpu-training-pilot, deepseek-238 (D84 centroid cache evaluator)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `2b08d79c4`.

## Goal

Move evaluation of sealed FBS query caches into the D84-authorized CPU module, correct
the matched-label control and enforce the registered pooled identity, power and
decision rules before the n3S capture is run.

## Cancel probe

Class: evaluation instrument. The GPU scorer already writes the cache and the existing
`readout_refit` module already owns exact integer refits and paired comparisons. The
new path only parses the sealed cache, fits from a stream, evaluates held-out panel
rows and emits a deterministic report. No trunk rerun or new model is required.

## Done when

1. `uor-r4-cpu-train` reads `uor-r4.fbs-query-cache/1`, verifies every manifest/file
   hash and seal, and refuses any query space other than the expected Q12 F32 encoding.
2. TRAIN centroid fitting is streaming: it never holds all 27.4M query vectors in RAM.
   It reports fitted rows and observations and enforces the registered power floor.
3. Candidate and control change the same eligible row set. Singleton count strata use
   their own centroid in both arms; a vacuous shuffle is `CONTROL_FAULT`.
4. Panel and pooled `(position,target)` row identities are computed and checked against
   registered expectations. The pooled candidate/original/control counts and paired
   verdicts are emitted by the Rust binary.
5. Two workers and one worker produce the same semantic report after timing/RSS fields
   are removed.
6. Focused tests cover cache tamper, singleton control, power, pooled identity, top-8
   preservation and deterministic worker order.

## Stop and ask if

Ask means decide, record and continue.

- Any change would alter the GPU scorer, training, serving or Claude's active path.
- The cache does not preserve the exact Q12 table/query bits needed by the integer
  evaluator.
- The laptop cannot meet the measured RAM bound; stop before opening a cache.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-centroid-*`
- `docs/loop/tasks/cpu-training-pilot/deepseek-238/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
