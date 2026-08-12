# AGENTS.md

## MUDBench Agent Entry Point

This repository implements **MUDBench**, a benchmark platform for evaluating autonomous agents in a persistent text-world environment.

Your job in this repository is **not** to invent the product.
Your job is to implement tasks inside the existing specification system.

Read this file first, then use the referenced documents as directed below.

---

## 1. Primary Operating Rules

You must follow these rules on every task:

- Work on **one task at a time**
- Stay strictly within the assigned task scope
- Do **not** modify files outside allowed task boundaries
- Do **not** modify root specification files unless explicitly instructed
- Do **not** introduce hidden behavior, hidden state, or undocumented shortcuts
- Prefer the smallest correct change over a broad refactor
- Run validation before finalizing work
- If the task is ambiguous, stop and surface the ambiguity instead of guessing

If you violate scope, the work is invalid even if the code “works.”

---

## 2. Mandatory Document Reading Order

For any implementation task, load documents in this order:

### Step 1: Project Identity
1. `PROJECT_CHARTER.md`
2. `PRODUCT_SPEC.md`

### Step 2: System Design
3. `SYSTEM_ARCHITECTURE.md`
4. `REPO_STRUCTURE.md`

### Step 3: Runtime + Execution Rules
5. `AGENT_RUNTIME_SPEC.md`
6. `CODING_AGENT_PLAYBOOK.md`
7. `TASK_TEMPLATE_LIBRARY.md`
8. `VALIDATION_AND_GUARDRAILS.md`
9. `CI_AND_AUTOMATION_PIPELINE.md`

### Step 4: Domain-Specific Specs
Load only the specs relevant to the task:

- world logic → `WORLD_SPEC.md`
- agent protocol → `AGENT_INTERFACE_SPEC.md`
- scoring → `BENCHMARK_SCORING_SPEC.md`
- replay / telemetry → `REPLAY_AND_TELEMETRY_SPEC.md`
- scenarios → `SCENARIO_SPEC.md`, `SCENARIO_LIBRARY.md`
- simulation core → `SIMULATION_ENGINE_SPEC.md`
- storage / schemas → `DATA_SCHEMA_AND_STORAGE_SPEC.md`
- experiment provenance → `EXPERIMENT_TRACKING_AND_REGISTRY.md`
- arena / tournaments → `ARENA_AND_TOURNAMENT_SYSTEM.md`
- matchmaking / ratings → `MATCHMAKING_AND_RATING_SYSTEM.md`
- reporting → `LEADERBOARD_AND_EVALUATION_REPORTING.md`
- baselines → `AGENT_BASELINE_LIBRARY.md`
- visualization → `VISUALIZATION_AND_ANALYTICS_SPEC.md`

Do **not** read every file by default if the task is narrow.
Use progressive disclosure and load only what is needed after the core operating files.

---

## 3. Task Intake Contract

Do not begin implementation unless the task includes:

- `task_id`
- `role`
- `objective`
- `scope`
- `allowed_files`
- `inputs`
- `constraints`
- `deliverables`
- `output_format`

If any of these are missing, request clarification or reject the task.

Use `TASK_TEMPLATE_LIBRARY.md` as the canonical source for task structure.

---

## 4. Write Boundaries

By default you may edit only:

- files explicitly listed in the task
- directly related tests
- directly related examples if requested

By default you may **not** edit:

- root specification files
- unrelated subsystem directories
- canonical scenario files
- CI / release / governance docs
- versioning docs
- registry history or replay artifacts

Repository boundaries are defined in `REPO_STRUCTURE.md`.

---

## 5. Required Validation Behavior

Before finalizing any task:

1. verify changed files remain in scope
2. run relevant tests
3. run deterministic validation if applicable
4. check replay compatibility if behavior affects simulation or scoring
5. confirm no undocumented behavior changes were introduced

If validation fails, do not claim success.

Validation rules are defined in:
- `VALIDATION_AND_GUARDRAILS.md`
- `CI_AND_AUTOMATION_PIPELINE.md`

---

## 6. Determinism Requirements

MUDBench is benchmark infrastructure.
Determinism is not optional.

You must avoid:

- uncontrolled randomness
- time-dependent behavior
- hidden global state
- nondeterministic ordering
- hidden scoring logic

If a change affects simulation, scoring, replay, or storage, treat determinism as a first-class requirement.

---

## 7. Runtime Behavior

Your execution model must follow `AGENT_RUNTIME_SPEC.md`.

High-level loop:

1. read task
2. assemble bounded context
3. inspect only relevant files
4. plan a minimal change
5. execute tool-mediated edits
6. run validation
7. inspect results
8. finalize or fail

Do not wander the repo without reason.

---

## 8. Baseline and Benchmark Awareness

When working on scoring, simulation, scenarios, or evaluation:

- preserve benchmark integrity
- preserve replay auditability
- preserve baseline comparability
- never change scoring or scenario meaning silently

Reference:
- `BENCHMARK_SCORING_SPEC.md`
- `SCENARIO_SPEC.md`
- `SCENARIO_LIBRARY.md`
- `AGENT_BASELINE_LIBRARY.md`

---

## 9. Output Expectations

Unless the task says otherwise, your final response should include:

- concise summary of what changed
- list of files changed
- validation/tests run
- unresolved issues or assumptions

Do not write long essays.
Do not describe work you did not actually complete.

---

## 10. Anti-Patterns

Never do the following:

- “helpfully” refactor unrelated code
- create large abstractions not required by the task
- hide important logic in `utils`
- modify task boundaries on your own
- skip tests because the result “looks right”
- silently change public interfaces
- silently update specs to match your implementation

That is not initiative.
That is repo vandalism with confidence.

---

## 11. Recommended Initial Commands

If the repository is already scaffolded, start with:

```bash
pwd
ls
git status
```

Then inspect only the relevant task files and referenced specs.

For validation, prefer project-native commands once available, such as:

```bash
pytest
make ci-local
```

Use only commands necessary for the assigned task.

---

## 12. Human Escalation Rule

If you encounter any of the following, stop and ask for clarification:

- ambiguous scope
- missing required files
- contradiction between specs
- required edit outside allowed files
- scoring or scenario behavior that appears inconsistent with benchmark intent

Do not improvise around these problems.

---

## 13. Final Principle

MUDBench is a benchmark system first.

That means:
- correctness beats speed
- auditability beats cleverness
- bounded change beats broad change
- reproducibility beats intuition

Implement accordingly.
