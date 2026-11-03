# REPORT: cpu-training-pilot, deepseek-99

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base:
`10e3956489bd6a5c593998c1fdc1f75634c04930`.

## What did not work, first

The merged deepseek-98 report retained a pre-rerun route figure in its
narrative even though its final result table and raw benchmark were correct.
That record mismatch is the defect this unit repairs.

## What changed

The deepseek-98 narrative now says route had a 2.6% advantage at 32 workers
instead of 0.9%. The conclusion is unchanged: 32 workers were not consistently
faster, while 16 physical workers captured most of the benefit. One existing
report line changed; no code, evidence, bar or result changed.

## Measurements

The retained deepseek-98 benchmark measured route at **5,192.435
candidate evaluations/s with 16 workers** and **5,327.637/s with 32 workers**,
a **2.61%** increase. All worker counts had checksum `66383767735728885`.

## Checks run at the head

- `loop report --check` for both the repair and affected deepseek-98 report.
- `loop pr-check --local`.
- Evidence `SHA256SUMS` validation.
- `git diff --check`.

## Skills used

No available Codex skill applied.

## Still open

The broader CPU program remains open under #3506; this unit only corrects the
merged performance record.

## Plain summary

The parallel scoring result is unchanged. A single percentage in the merged
write-up was left from an earlier benchmark run, so this correction makes the
text agree with the final retained table: route was 2.6% faster at 32 workers,
not 0.9%.
