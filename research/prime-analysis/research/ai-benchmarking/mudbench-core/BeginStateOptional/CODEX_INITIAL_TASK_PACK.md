# MUDBench Initial Task Pack for Codex

## Purpose

This file gives you a practical sequence for onboarding Codex into MUDBench and then assigning the first real implementation tasks.

Use these prompts in order.

---

# Task 0 — Repository Ingestion

## Prompt

```text
You are working inside the MUDBench repository.

Start by reading AGENTS.md and follow it strictly.

Then do the following in order:

1. Read the core operating documents referenced in AGENTS.md:
   - PROJECT_CHARTER.md
   - PRODUCT_SPEC.md
   - SYSTEM_ARCHITECTURE.md
   - REPO_STRUCTURE.md
   - AGENT_RUNTIME_SPEC.md
   - CODING_AGENT_PLAYBOOK.md
   - TASK_TEMPLATE_LIBRARY.md
   - VALIDATION_AND_GUARDRAILS.md
   - CI_AND_AUTOMATION_PIPELINE.md

2. Build a concise internal understanding of:
   - project mission
   - subsystem boundaries
   - task execution rules
   - validation requirements
   - repo write boundaries

3. Do NOT implement code yet.

4. Produce a short repo-ingestion report containing:
   - the core purpose of MUDBench
   - the major subsystems
   - the execution rules you must obey
   - the validation pipeline you must respect
   - the files you should treat as governance/specification files
   - a proposed initial implementation plan aligned to IMPLEMENTATION_PHASES.md

5. Keep the report concise and structured.

Important constraints:
- Do not modify files
- Do not invent requirements
- Do not read every spec unless needed beyond the core reading list
- Use progressive disclosure after the initial core docs
```

---

# Task 1 — Phase 0 Repository Scaffolding

## Prompt

```text
Read AGENTS.md and follow it strictly.

Task:
Implement Phase 0 repository scaffolding for MUDBench.

Use:
- IMPLEMENTATION_PHASES.md
- REPO_STRUCTURE.md
- CODING_AGENT_PLAYBOOK.md
- TASK_TEMPLATE_LIBRARY.md
- VALIDATION_AND_GUARDRAILS.md

Constraints:
- only create the directories and placeholder files required for Phase 0
- do not modify specification documents
- keep the implementation minimal
- do not introduce production code beyond what is needed for scaffolding
- provide a concise summary of files created
- run any local validation that exists

Success criteria:
- repository structure exists per spec
- placeholder package/module layout exists
- minimal CLI entry point exists if appropriate
- no root spec files are edited
```

---

# Task 2 — Simulation Engine Skeleton

## Prompt

```text
Read AGENTS.md and follow it strictly.

Task:
Create the initial simulation engine skeleton for MUDBench.

Use:
- SIMULATION_ENGINE_SPEC.md
- SYSTEM_ARCHITECTURE.md
- WORLD_SPEC.md
- REPO_STRUCTURE.md
- CODING_AGENT_PLAYBOOK.md
- VALIDATION_AND_GUARDRAILS.md

Constraints:
- only create foundational engine files and tests
- do not implement full game logic yet
- create deterministic interfaces and clean module boundaries
- do not modify scoring, replay, or scenario files beyond what is strictly needed for interfaces
- provide a concise summary of files changed and validations run

Success criteria:
- simulation controller skeleton exists
- world state manager interface exists
- action processor interface exists
- event logger interface exists
- deterministic test scaffolding exists
```

---

# Task 3 — Agent Interface Skeleton

## Prompt

```text
Read AGENTS.md and follow it strictly.

Task:
Implement the initial agent interface skeleton for MUDBench.

Use:
- AGENT_INTERFACE_SPEC.md
- SYSTEM_ARCHITECTURE.md
- REPO_STRUCTURE.md
- CODING_AGENT_PLAYBOOK.md
- VALIDATION_AND_GUARDRAILS.md

Constraints:
- implement only the basic protocol and runners
- support local process mode first
- HTTP mode may be stubbed if needed
- no broad refactors
- add tests for observation/action schema handling

Success criteria:
- structured observation model exists
- action submission model exists
- local process runner exists
- validation tests exist
```

---

# Task 4 — First Baseline Agents

## Prompt

```text
Read AGENTS.md and follow it strictly.

Task:
Implement the first baseline agents for MUDBench.

Use:
- AGENT_BASELINE_LIBRARY.md
- AGENT_INTERFACE_SPEC.md
- REPO_STRUCTURE.md
- CODING_AGENT_PLAYBOOK.md
- VALIDATION_AND_GUARDRAILS.md

Constraints:
- implement random baseline and simple greedy baseline first
- keep both deterministic under seed
- do not access hidden state
- place code in the examples/baselines or designated baseline location per repo structure
- add simple tests where appropriate

Success criteria:
- random baseline agent exists
- greedy baseline agent exists
- both conform to the interface spec
- both are documented briefly
```

---

# Task 5 — Validation Hook Wiring

## Prompt

```text
Read AGENTS.md and follow it strictly.

Task:
Wire minimal validation hooks into the local developer workflow.

Use:
- VALIDATION_AND_GUARDRAILS.md
- CI_AND_AUTOMATION_PIPELINE.md
- REPO_STRUCTURE.md
- DEVELOPER_GUIDE.md

Constraints:
- create minimal local validation commands only
- do not overbuild CI
- prefer small scripts or make targets if appropriate
- do not invent release automation yet

Success criteria:
- local test command exists
- local lint/validation command exists if appropriate
- deterministic validation placeholder exists
- documentation is updated only if directly required
```

---

# Recommended Working Order

1. Task 0 — Ingestion
2. Task 1 — Phase 0 scaffolding
3. Task 2 — Simulation engine skeleton
4. Task 3 — Agent interface skeleton
5. Task 4 — Baseline agents
6. Task 5 — Validation hook wiring

Do not skip straight to full scenario logic or tournament systems.
That would be classic human behavior: ambitious, chaotic, and expensive.
