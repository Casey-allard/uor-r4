# ORDER: cpu-training-pilot, deepseek-249 (final review repair)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `3f0b0fb55`. Parent ORDER: [deepseek-248](../deepseek-248/ORDER.md).

## Goal

Repair the three residual findings from the second fresh review:

1. Document that archive validation failure aborts before report generation,
   unlike a checkpoint source failure, which is written into the report.
2. Add the 18,191 bytes of `SHA256SUMS` and `.list` metadata downloaded beside
   the two archives, making the complete run-root input footprint 556,424,899
   bytes.
3. Set the resource ledger's `updated_utc` after the final measurements it
   records.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-247/README.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-247/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-248/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-249/**`
- `docs/labs/cpu-training-pilot-real-ask-regression-2026-11-09/resource-ledger.json`

## Done when

1. README's archive and checkpoint failure behavior matches the binary.
2. The ledger's temporary input total includes all downloaded run-root files.
3. The ledger timestamp follows its measurements.
4. Report policy, local policy, full gates and a third fresh exact-head review
   pass.

## Stop and ask if

Do not change executable code, accepted checkpoint metrics, manifest hashes,
evidence report hashes, Claude-owned files, serving or GPU state.
