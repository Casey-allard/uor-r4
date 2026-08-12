# MUDBench Task Template Library

## Document Status
Version: 0.1
Status: Draft
Purpose: Provide standardized, reusable task templates for coding agents to ensure consistency, safety, and alignment with MUDBench architecture and benchmark integrity.

---

# 1. Why This Exists

You already defined how agents behave (see Coding Agent Playbook). 
This document defines how you *talk to them*.

Without templates:
- tasks drift
- scope expands
- agents improvise
- repo structure degrades

With templates:
- tasks are deterministic
- outputs are comparable
- agents stay constrained

This is not optional.

---

# 2. Core Design Principles

All task templates must:

- be explicit
- be bounded
- be reproducible
- align with subsystem boundaries
- map cleanly to implementation phases

Tasks are contracts, not suggestions.

---

# 3. Canonical Task Structure

All templates must follow this schema:

```json
{
  "task_id": "string",
  "role": "builder_agent | test_agent | refactor_agent | spec_follower_agent | worker_agent",
  "objective": "clear, single-purpose goal",
  "scope": ["list of modules"],
  "allowed_files": ["explicit file paths"],
  "inputs": ["specs or files"],
  "constraints": ["rules the agent must follow"],
  "deliverables": ["expected outputs"],
  "output_format": "strict output requirement"
}
```

If a task deviates from this, expect unpredictable results.

---

# 4. Template Categories

## 4.1 Feature Implementation (Builder)

```json
{
  "task_id": "feature_<name>_v1",
  "role": "builder_agent",
  "objective": "Implement <feature>",
  "scope": ["src/<module>/"],
  "allowed_files": [
    "src/<module>/<file>.py",
    "tests/unit/test_<feature>.py"
  ],
  "inputs": [
    "<SPEC_FILE>.md"
  ],
  "constraints": [
    "must be deterministic",
    "must not modify other subsystems"
  ],
  "deliverables": [
    "working implementation",
    "unit tests"
  ],
  "output_format": "code + brief summary"
}
```

Use this for:
- movement
- combat
- scoring modules
- protocol handlers

---

## 4.2 Test Generation (Test Agent)

```json
{
  "task_id": "test_<module>_v1",
  "role": "test_agent",
  "objective": "Create tests for <module>",
  "scope": ["tests/unit/"],
  "allowed_files": [
    "tests/unit/test_<module>.py"
  ],
  "inputs": [
    "src/<module>/<file>.py"
  ],
  "constraints": [
    "must cover edge cases",
    "must verify determinism"
  ],
  "deliverables": [
    "unit tests"
  ],
  "output_format": "code only"
}
```

---

## 4.3 Spec Implementation (Spec-Follower)

```json
{
  "task_id": "spec_<component>_v1",
  "role": "spec_follower_agent",
  "objective": "Implement behavior defined in spec",
  "scope": ["src/<component>/"],
  "allowed_files": [
    "src/<component>/<file>.py"
  ],
  "inputs": [
    "<SPEC_FILE>.md"
  ],
  "constraints": [
    "must match spec exactly",
    "no additional features"
  ],
  "deliverables": [
    "implementation aligned with spec"
  ],
  "output_format": "code + mapping to spec sections"
}
```

---

## 4.4 Refactor Task

```json
{
  "task_id": "refactor_<module>_v1",
  "role": "refactor_agent",
  "objective": "Improve structure of <module>",
  "scope": ["src/<module>/"],
  "allowed_files": [
    "src/<module>/"
  ],
  "inputs": [
    "existing codebase"
  ],
  "constraints": [
    "no behavior change",
    "no new features"
  ],
  "deliverables": [
    "cleaner code",
    "updated tests if needed"
  ],
  "output_format": "diff + summary"
}
```

---

## 4.5 Worker Task (Atomic Execution)

```json
{
  "task_id": "worker_<operation>_v1",
  "role": "worker_agent",
  "objective": "Perform a narrowly scoped operation",
  "scope": ["single file or function"],
  "allowed_files": [
    "<specific file>"
  ],
  "inputs": [
    "explicit data or config"
  ],
  "constraints": [
    "no side effects outside scope"
  ],
  "deliverables": [
    "exact output"
  ],
  "output_format": "STRICT_JSON_ONLY"
}
```

Use this for:
- data transforms
- scoring calculations
- deterministic checks

---

## 4.6 Scenario Creation

```json
{
  "task_id": "scenario_<name>_v1",
  "role": "builder_agent",
  "objective": "Create new benchmark scenario",
  "scope": ["scenarios/"],
  "allowed_files": [
    "scenarios/canonical/<scenario>.json"
  ],
  "inputs": [
    "SCENARIO_SPEC.md",
    "SCENARIO_LIBRARY.md"
  ],
  "constraints": [
    "must be deterministic",
    "must align with scoring signals"
  ],
  "deliverables": [
    "scenario definition"
  ],
  "output_format": "json + summary"
}
```

---

## 4.7 Evaluation Metric Implementation

```json
{
  "task_id": "metric_<name>_v1",
  "role": "builder_agent",
  "objective": "Implement scoring metric",
  "scope": ["src/evaluation/"],
  "allowed_files": [
    "src/evaluation/metrics/<metric>.py",
    "tests/unit/test_<metric>.py"
  ],
  "inputs": [
    "BENCHMARK_SCORING_SPEC.md"
  ],
  "constraints": [
    "must be replay-auditable",
    "no hidden state"
  ],
  "deliverables": [
    "metric implementation",
    "tests"
  ],
  "output_format": "code + summary"
}
```

---

# 5. Anti-Patterns (Do Not Do This)

Bad task:

- vague objective
- no file boundaries
- implicit requirements
- missing constraints

Example of failure:

"Implement combat system"  
(no scope, no files, no constraints, no tests)

That’s not a task. That’s a gamble.

---

# 6. Composition Strategy

Complex features should be broken into multiple tasks:

Example:

- movement_core_v1
- movement_validation_v1
- movement_tests_v1

Do not assign large multi-system tasks.

Agents are fast. That does not mean they are wise.

---

# 7. Phase Alignment

Tasks must align with implementation phases:

- Phase 0 → repo scaffolding tasks
- Phase 1 → world mechanics tasks
- Phase 2 → agent interface tasks
- Phase 3 → evaluation tasks
- Phase 4 → replay/telemetry tasks
- Phase 5 → arena tasks

If a task crosses phases, it is probably wrong.

---

# 8. Output Enforcement

Strict output formats reduce ambiguity:

- code only
- code + summary
- json only
- diff format

If you do not enforce output, the agent will improvise.

And you will regret that.

---

# 9. Versioning

Every task must be versioned:

- v1 → initial
- v2 → revision
- v3 → refinement

Never silently mutate tasks.

---

# 10. Closing Statement

You built:
- a spec system
- an architecture
- a playbook

This document is the execution layer that turns all of that into actual work.

Without it, your agents will behave like very confident interns.

With it, they behave like constrained, reliable systems.

Choose wisely.
