# Performance Baseline Report

## Objective
Track runtime hotspots and improvements from PR-2 changes.

## Baseline Method
- Run `--fast_dev 1` baseline route.
- Record per-phase timings from JSON summary.

## Hotspots to track
- chart optimization
- routing eval
- growth splitting

## Current Status
- First instrumented smoke run recorded (`run_tag=smoke_single`):
  - `dataset`: 0.0005s
  - `chart_opt`: 0.0838s
  - `routing_eval`: 0.0020s
  - `training_ema`: 0.0244s
  - `growth`: 0.1258s
  - `total`: 0.2485s
- Staged sweep executed with 28 runs via `runs/run_pipeline.sh`.
- Current hotspot trend under fast-dev: growth + chart optimization dominate wall-clock.
