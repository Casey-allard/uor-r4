# Project Map

This is the top-level orientation page for the `AI-Research` monorepo.

## Monorepo layout

- [README.md](/Users/adminamn/AI-Research/README.md)
  High-level description of the combined repository.
- [ai-router](/Users/adminamn/AI-Research/ai-router)
  Geometric routing research, transport laws, and router experiments.
- [MUDBench](/Users/adminamn/AI-Research/MUDBench)
  Benchmark, simulation, and shared-world infrastructure for agents and human players.
- [ramsey](/Users/adminamn/AI-Research/ramsey)
  Smaller mathematical experiment harness for recursive Ramsey-style studies.

## Best Starting Points

### `ai-router`

- [AGENTS.md](/Users/adminamn/AI-Research/ai-router/AGENTS.md)
  Start here for repo-specific working rules.
- [Makefile](/Users/adminamn/AI-Research/ai-router/Makefile)
  Entry point for the documented startup checks.
- [router-research](/Users/adminamn/AI-Research/ai-router/router-research)
  Main research code, docs, and experiment outputs.
- [router-research/docs/research/KILL_LIST_TRACKER.md](/Users/adminamn/AI-Research/ai-router/router-research/docs/research/KILL_LIST_TRACKER.md)
  Central status board for the staged research program.
- [router-research/docs/research/ACTIVE_STATE.md](/Users/adminamn/AI-Research/ai-router/router-research/docs/research/ACTIVE_STATE.md)
  Current branch context and active research state.
- [paper1_repo_inventory.md](/Users/adminamn/AI-Research/ai-router/paper1_repo_inventory.md)
  Useful index for the paper/manuscript side of the router project.

### `MUDBench`

- [AGENTS.md](/Users/adminamn/AI-Research/MUDBench/AGENTS.md)
  Start here for repo-specific execution and scope rules.
- [README.md](/Users/adminamn/AI-Research/MUDBench/README.md)
  Project overview for the benchmark/game-world system.
- [PROJECT_CHARTER.md](/Users/adminamn/AI-Research/MUDBench/PROJECT_CHARTER.md)
  Top-level project identity.
- [PRODUCT_SPEC.md](/Users/adminamn/AI-Research/MUDBench/PRODUCT_SPEC.md)
  Core specification for the platform.
- [SYSTEM_ARCHITECTURE.md](/Users/adminamn/AI-Research/MUDBench/SYSTEM_ARCHITECTURE.md)
  Main system design reference.
- [src](/Users/adminamn/AI-Research/MUDBench/src)
  Runtime implementation.
- [tests](/Users/adminamn/AI-Research/MUDBench/tests)
  Unit and integration coverage.
- [scenarios](/Users/adminamn/AI-Research/MUDBench/scenarios)
  Canonical benchmark and world scenarios.

### `ramsey`

- [AGENTS.md](/Users/adminamn/AI-Research/ramsey/AGENTS.md)
  Repo-specific research rules.
- [scripts](/Users/adminamn/AI-Research/ramsey/scripts)
  Experiment scripts.
- [results](/Users/adminamn/AI-Research/ramsey/results)
  Tables, reports, and saved experiment outputs.

## Root-Level Research Artifacts

The repository root contains shared research outputs that sit above any one
subproject, including:

- cross-project notes and summaries
- experiment CSVs
- generated figures
- small standalone scripts such as
  [c2_shared_astar_unseen_wheel_test.py](/Users/adminamn/AI-Research/c2_shared_astar_unseen_wheel_test.py)

## Working Conventions

- Treat `/Users/adminamn/AI-Research` as the canonical repo root.
- `ai-router/`, `MUDBench/`, and `ramsey/` are now ordinary folders inside the
  monorepo, not nested git repositories.
- Root-level ignores intentionally leave local scratch data such as
  `MUDBench/tmp/` out of version control.

## Suggested Reading Order

If the task is broad and spans the whole project:

1. [README.md](/Users/adminamn/AI-Research/README.md)
2. [PROJECT_MAP.md](/Users/adminamn/AI-Research/PROJECT_MAP.md)
3. The relevant subproject `AGENTS.md`
4. The relevant project charter / architecture / active-state docs

If the task is narrow, start in the appropriate subproject and avoid loading the
rest of the monorepo unless it is directly relevant.
