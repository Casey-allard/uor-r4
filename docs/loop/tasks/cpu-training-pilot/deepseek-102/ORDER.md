# ORDER: cpu-training-pilot, deepseek-102 (correct worker-state comparison)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `6729e5d507bd39ef400c6515bc29b75f97e682d7`.

## Goal

Repair the fresh-context CHANGES review of PR #3620 at
`b8e01b7297a708f933d8ec95e2dcfabb9d146892`.

The reviewed timing implementation and test are accepted. Two retained records
remain wrong: their semantic-comparison command includes the worker-specific
served-audit path, so it returns four hashes instead of one, and the active
report's audit range omits the reverse-order readings.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-100/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-101/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-102/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. The comparison command excludes only the worker-specific audit path while
   retaining the stable audit hash, parity and workload fields.
2. Executing the documented command returns exactly `1`.
3. Every timing range is labeled by its run subset or spans all eight runs.
4. All summaries and the active deepseek-102 REPORT agree with the raw evidence.
5. The exact PR diff, evidence checksums, report check, local PR check and full
   gate suite pass.
6. A fresh exact-head review approves the final head.

## Stop and ask

- Stop if a fix changes timing evidence, an artifact, a panel reading or the
  worker recommendation.
- Do not alter the already accepted timing implementation or test.
