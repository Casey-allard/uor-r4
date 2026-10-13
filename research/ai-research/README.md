# AI-Research

**Imported research for the UOR-R4 Geometric Language Model.** This directory
is a preserved AI-Research snapshot, with its original project description
below. Use the [research index](../README.md), [project map](../../docs/PROJECT_MAP.md),
[canonical plan](../../docs/integration/project-track.md),
[current-state](../../docs/integration/current-state.md) and
[model direction](../../docs/integration/model-direction-2026-09.md) for the
current Rust model. Nested experiments and duplicate snapshots retain their
provenance; their internal "canonical" labels apply to their historical source
repository. They do not qualify general prose completion or general reasoning.

This repository is the root research workspace for the broader project that now
combines:

- `ai-router/` — geometric routing research and experiments
- `MUDBench/` — benchmark and game-world infrastructure for agent evaluation
- `ramsey/` — smaller mathematical research experiments

It also contains root-level notes, figures, scripts, and experimental artifacts
that cut across those subprojects.

Start with [PROJECT_MAP.md](PROJECT_MAP.md) for a
quick orientation to the monorepo.

## Structure

- `ai-router/`
  Geometry-native routing research, transport laws, and router experiments.
- `MUDBench/`
  Benchmark, simulation, and world/runtime infrastructure.
- `ramsey/`
  Local mathematical experiment harness for recursive Ramsey-style studies.
- repository root
  Shared notes, plots, CSV outputs, and cross-project research artifacts.

## Notes

- This is the new canonical monorepo for the combined project.
- Some large research artifacts remain in version control where they were
  already part of prior project history.
- Local scratch outputs such as `MUDBench/tmp/` are intentionally ignored at the
  root level.
