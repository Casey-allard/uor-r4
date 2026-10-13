# MUDBench Repository Structure

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the canonical repository layout, module boundaries, ownership expectations, and file placement rules for MUDBench.

---

# 1. Purpose of This Document

MUDBench is intended to be developed in a modular, benchmark-first way. The project charter requires clear module boundaries and a documented repository structure so the system can support scientific use and safe agent-led development. fileciteturn9file0

This document defines:

- the canonical top-level repository layout
- where each subsystem belongs
- how documentation should be organized
- what coding agents are allowed to modify
- how to avoid structural drift as the project grows

This file is a governance document, not just a convenience guide.

---

# 2. Repository Design Principles

The repository structure must reflect the project’s architectural and product goals.

The structure should be:

- modular
- easy to navigate
- aligned with subsystem boundaries
- stable across early development
- safe for parallel work by humans and coding agents

The repository must favor clarity over cleverness.

---

# 3. Canonical Top-Level Layout

The canonical MUDBench repository should use the following top-level structure:

```text
mudbench/
├─ README.md
├─ PROJECT_CHARTER.md
├─ PRODUCT_SPEC.md
├─ SYSTEM_ARCHITECTURE.md
├─ WORLD_SPEC.md
├─ AGENT_INTERFACE_SPEC.md
├─ BENCHMARK_SCORING_SPEC.md
├─ REPLAY_AND_TELEMETRY_SPEC.md
├─ IMPLEMENTATION_PHASES.md
├─ DEVELOPER_GUIDE.md
├─ SCENARIO_SPEC.md
├─ SCENARIO_LIBRARY.md
├─ REPO_STRUCTURE.md
│
├─ docs/
├─ src/
├─ scenarios/
├─ tests/
├─ tools/
├─ scripts/
├─ replay_data/
├─ configs/
└─ examples/
```

The top-level markdown files define the planning and governance layer.
Implementation code and runtime assets belong in the subdirectories.

---

# 4. Top-Level File Policy

Top-level files should be reserved for:

- project governance documents
- core specifications
- repository onboarding
- implementation coordination references

Top-level files should not be used for:

- experimental notes
- one-off logs
- temporary design scraps
- test outputs
- ad hoc agent instructions

This prevents the root directory from turning into a landfill of abandoned intentions.

---

# 5. docs/ Directory

Purpose:

Store secondary documentation that supports contributors but is not part of the canonical top-level specification set.

Recommended layout:

```text
docs/
├─ decisions/
├─ onboarding/
├─ research/
├─ benchmark_notes/
└─ archive/
```

Use cases:

- architecture decision records
- contributor onboarding guides
- research notes
- historical drafts
- deprecated specifications

Rules:

- final canonical specs stay at repository root unless deliberately migrated later
- stale docs should be archived, not left mixed with active guidance

---

# 6. src/ Directory

Purpose:

Store all production source code for the MUDBench platform.

Recommended layout:

```text
src/
├─ core/
├─ world/
├─ agents/
├─ evaluation/
├─ replay/
├─ arena/
├─ scenarios/
├─ api/
├─ cli/
└─ utils/
```

Each directory maps to a subsystem defined in the architecture. The architecture establishes the core subsystems as Agent Gateway, Simulation Controller, World Engine, Evaluation Engine, and Replay/Telemetry. fileciteturn9file3

---

# 7. src/core/

Purpose:

Store the simulation controller and low-level shared engine infrastructure.

Typical contents:

- simulation loop
- event queue
- world state manager
- scheduler
- seed control
- run lifecycle manager

Rules:

- `core/` should not contain world content definitions
- `core/` should not contain benchmark scoring logic directly
- `core/` should remain as domain-agnostic as possible

---

# 8. src/world/

Purpose:

Store world-model logic defined by the world specification. The world spec defines rooms, regions, entities, NPCs, items, hazards, inventory, movement, combat, and quests. fileciteturn9file4

Recommended layout:

```text
src/world/
├─ rooms/
├─ entities/
├─ npcs/
├─ items/
├─ hazards/
├─ quests/
├─ combat/
└─ state/
```

Rules:

- world mechanics live here
- no replay formatting logic
- no score aggregation logic
- no network protocol code

---

# 9. src/agents/

Purpose:

Store the agent-facing protocol implementation and SDK helpers.

Recommended layout:

```text
src/agents/
├─ gateway/
├─ local_runner/
├─ http_runner/
├─ protocol/
└─ sdk/
```

This directory implements the observation/action contract defined in the agent interface specification. fileciteturn9file5

Rules:

- protocol schemas belong here
- agent transport logic belongs here
- internal benchmark scoring must not be embedded here

---

# 10. src/evaluation/

Purpose:

Store scoring logic, metric calculators, score aggregation, and benchmark lifecycle hooks.

Recommended layout:

```text
src/evaluation/
├─ metrics/
├─ scoring/
├─ normalization/
├─ scorecards/
└─ benchmark_runner/
```

This directory implements the logic defined in the benchmark scoring spec. fileciteturn9file6

Rules:

- evaluation consumes world signals but does not own world state
- evaluation must remain replay-auditable
- hidden scoring shortcuts are prohibited

---

# 11. src/replay/

Purpose:

Store replay logging, telemetry, playback reconstruction, and related utilities.

Recommended layout:

```text
src/replay/
├─ logging/
├─ telemetry/
├─ reconstruction/
├─ integrity/
└─ viewer_support/
```

This directory implements the replay and telemetry specification. fileciteturn9file7

Rules:

- replay must record, not mutate, simulation behavior
- telemetry collection must not alter benchmark outcomes

---

# 12. src/arena/

Purpose:

Store persistent arena logic and competition-layer systems added after benchmark mode is stable.

Typical contents:

- persistent world services
- ladder history
- matchmaking
- run history persistence
- season logic

Rules:

- arena-specific persistence must not leak into official benchmark mode
- benchmark mode and arena mode should share engine components but not mutable competitive state

---

# 13. src/scenarios/

Purpose:

Store code used to load, validate, and execute scenario definitions.

Typical contents:

- scenario registry
- validation utilities
- scenario loader
- scenario adapters

This directory should implement the scenario structure defined in the scenario specification and the canonical entries defined in the scenario library. fileciteturn9file10 fileciteturn9file11

---

# 14. src/api/

Purpose:

Store any external API surfaces needed for remote participation or hosted deployment.

Typical contents:

- REST endpoints
- auth stubs
- submission handlers
- run metadata endpoints

Rules:

- do not place core simulation logic here
- the API layer should wrap internal modules, not replace them

---

# 15. src/cli/

Purpose:

Store command-line interfaces for local development and benchmark execution.

Typical commands may include:

- `mudbench run`
- `mudbench replay`
- `mudbench validate-scenario`
- `mudbench score-run`

Rules:

- CLI code should call into stable internal modules
- business logic should not be embedded in command handlers

---

# 16. src/utils/

Purpose:

Store shared utility code that does not belong clearly to another subsystem.

Allowed examples:

- serialization helpers
- seed helpers
- validation primitives
- path utilities

Disallowed examples:

- dumping major domain logic into utils because no one wanted to think carefully

If a utility becomes domain-specific, move it into the proper subsystem.

---

# 17. scenarios/ Directory

Purpose:

Store benchmark scenario definitions and scenario assets.

Recommended layout:

```text
scenarios/
├─ canonical/
├─ experimental/
├─ validation/
└─ archive/
```

Suggested contents:

- scenario JSON/YAML files
- map definitions
- scenario metadata
- validation fixtures

Rules:

- `canonical/` contains official benchmark scenarios
- `experimental/` contains trial scenarios not yet benchmark-approved
- `archive/` stores deprecated scenarios

This directory stores content; `src/scenarios/` stores code.

---

# 18. tests/ Directory

Purpose:

Store automated tests for all subsystems.

Recommended layout:

```text
tests/
├─ unit/
├─ integration/
├─ benchmark/
├─ replay/
└─ fixtures/
```

Testing expectations are reinforced by the implementation phases, which require automated tests and deterministic validation gates. fileciteturn9file8

Rules:

- each subsystem should have unit tests
- cross-subsystem behavior should have integration tests
- deterministic replay should have dedicated tests
- fixtures must remain stable and documented

---

# 19. tools/ Directory

Purpose:

Store developer utilities that are not part of the main runtime.

Examples:

- scenario lint tools
- replay inspection helpers
- benchmark report generators
- data cleanup utilities

Rules:

- tools may depend on production code
- production code should not depend on tools

---

# 20. scripts/ Directory

Purpose:

Store shell scripts and short automation tasks for local workflows.

Examples:

- local bootstrap
- test runners
- formatting helpers
- CI setup wrappers

Rules:

- scripts should be thin wrappers around documented commands
- long-term logic belongs in `src/` or `tools/`, not in random shell fragments

---

# 21. replay_data/ Directory

Purpose:

Store local replay outputs and development run artifacts.

Recommended layout:

```text
replay_data/
├─ local/
├─ benchmark/
├─ arena/
└─ archived/
```

Rules:

- do not commit bulky replay artifacts unless explicitly intended
- use this directory for local inspection and structured storage conventions
- production deployments may redirect replay storage elsewhere

---

# 22. configs/ Directory

Purpose:

Store static configuration for environments and runtime defaults.

Examples:

- benchmark defaults
- local development settings
- logging configuration
- environment-specific overrides

Rules:

- secrets must not be stored here
- config files should be human-readable and version-controlled where appropriate

---

# 23. examples/ Directory

Purpose:

Store example agents, example scenario configs, and developer reference materials.

Recommended layout:

```text
examples/
├─ agents/
├─ scenarios/
└─ notebooks/
```

Examples help bridge the gap between the specifications and practical developer use. The developer guide already points toward example agents and scenario usage, so this directory formalizes where they live. fileciteturn9file9

---

# 24. Coding-Agent Write Boundaries

To keep agent-led development safe, coding agents should follow these write constraints:

## Allowed by default
- files inside explicitly assigned `src/` submodules
- test files paired to assigned modules
- documentation files explicitly named in the task
- example files in `examples/`

## Not allowed by default
- root-level specification files
- unrelated subsystem directories
- scenario files outside assigned scope
- CI or deployment config unless assigned
- historical or archive directories

Coding agents should not reorganize directories unless the task explicitly requests repository restructuring.

---

# 25. Ownership Guidance

Recommended ownership model:

| Area | Primary Owner Type |
|---|---|
| root specs | human lead / architecture lead |
| src/core | engine lead |
| src/world | world systems lead |
| src/agents | interface / SDK lead |
| src/evaluation | benchmark lead |
| src/replay | telemetry / replay lead |
| scenarios/canonical | benchmark governance lead |
| tests/benchmark | validation lead |

This does not require formal teams at v0, but it provides a sane mental model for task assignment.

---

# 26. Naming Conventions

Repository naming conventions should be simple and consistent.

Rules:

- directories use lowercase with underscores only if needed
- markdown specs use uppercase descriptive filenames when canonical
- Python modules use lowercase snake_case
- scenario IDs use lowercase with version suffixes, such as `forest_retrieval_v1`

Consistency matters more than aesthetic preferences. Humans love pretending these are separate things.

---

# 27. Archive Policy

Deprecated or replaced material should be moved, not silently deleted when possible.

Recommended archive locations:

- `docs/archive/`
- `scenarios/archive/`
- `replay_data/archived/`

This preserves project history without polluting active directories.

---

# 28. Initial Repository Target State

Once the current documentation pass is complete, the repository should minimally look like:

```text
mudbench/
├─ README.md
├─ PROJECT_CHARTER.md
├─ PRODUCT_SPEC.md
├─ SYSTEM_ARCHITECTURE.md
├─ WORLD_SPEC.md
├─ AGENT_INTERFACE_SPEC.md
├─ BENCHMARK_SCORING_SPEC.md
├─ REPLAY_AND_TELEMETRY_SPEC.md
├─ IMPLEMENTATION_PHASES.md
├─ DEVELOPER_GUIDE.md
├─ SCENARIO_SPEC.md
├─ SCENARIO_LIBRARY.md
├─ REPO_STRUCTURE.md
├─ docs/
├─ src/
├─ scenarios/
├─ tests/
├─ tools/
├─ scripts/
├─ replay_data/
├─ configs/
└─ examples/
```

This target state is enough to begin implementation without structural ambiguity.

---

# 29. Relationship to Future Documents

This repository structure should later be complemented by:

- coding-agent execution manuals
- task templates
- scenario validation guides
- CI and release documentation
- contribution and review rules

This file defines where things live.
Other documents define how work moves through those places.

---

# 30. Closing Statement

A clean repository is part of benchmark integrity.

If the codebase structure is unstable, the implementation becomes unstable.
If the implementation becomes unstable, the benchmark becomes untrustworthy.

MUDBench should be organized so that humans and coding agents can both navigate it without improvising new filesystem theology every week.
