# ORDER: cpu-training-pilot, deepseek-99 (correct the parallel benchmark record)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `10e3956489bd6a5c593998c1fdc1f75634c04930`.

## Goal

Correct the merged deepseek-98 narrative where it says the 32-worker route
result had only a 0.9% advantage. The final retained benchmark measured
5,327.637/s at 32 workers against 5,192.435/s at 16, a 2.6% advantage. The
substantive conclusion is unchanged: 32 workers did not consistently beat 16.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-99/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-98/REPORT.md`

## Done when

1. The deepseek-98 REPORT states the measured 2.6% route difference.
2. The evidence manifest still validates.
3. Report and local PR checks pass.
4. The correction merges and fresh main is verified.

## Stop if

- The retained raw benchmark does not contain the quoted values.
- The correction would change code, evidence or the reported conclusion.
