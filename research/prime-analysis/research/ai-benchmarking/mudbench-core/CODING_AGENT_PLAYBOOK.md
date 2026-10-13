# MUDBench Coding Agent Playbook

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how coding agents (e.g., Codex, GPT-based agents, or other autonomous builders) operate safely, predictably, and productively within the MUDBench repository.

---

# 1. Purpose of This Document

MUDBench is explicitly designed to support agent-led development.

This playbook defines:

- how coding agents receive tasks
- what they are allowed to modify
- how they report work
- how to prevent structural or architectural damage
- how to ensure alignment with benchmark-first principles

Without this document, agents will eventually “optimize” the repository into chaos.

---

# 2. Core Principle

Coding agents are not creative directors.

They are constrained executors operating within a defined system.

They must:

- respect architecture boundaries
- follow task specifications exactly
- avoid introducing undocumented behavior
- prioritize correctness over cleverness

The project charter explicitly prioritizes benchmark integrity and modularity over novelty. fileciteturn10file0

---

# 3. Agent Roles

Agents may operate in different roles:

## 3.1 Builder Agent
Implements features inside assigned modules.

## 3.2 Refactor Agent
Improves code structure within a defined scope.

## 3.3 Test Agent
Writes or updates tests for existing functionality.

## 3.4 Spec-Follower Agent
Implements behavior directly from a specification document.

## 3.5 Worker Agent
Executes a narrowly scoped task with strict input/output requirements.

Agents must not switch roles mid-task.

---

# 4. Task Format (Canonical)

All agent tasks must follow a structured format.

Example:

```json
{
  "task_id": "world_move_v1",
  "role": "builder_agent",
  "objective": "Implement movement system between rooms",
  "scope": [
    "src/world/movement.py"
  ],
  "allowed_files": [
    "src/world/movement.py",
    "tests/unit/test_movement.py"
  ],
  "inputs": [
    "WORLD_SPEC.md"
  ],
  "constraints": [
    "must be deterministic",
    "must not modify other subsystems"
  ],
  "deliverables": [
    "movement logic",
    "unit tests"
  ],
  "output_format": "code + brief summary"
}
```

Agents must not proceed if the task is ambiguous.

---

# 5. File Access Rules

The repository structure defines strict write boundaries. fileciteturn10file12

## 5.1 Allowed

- files explicitly listed in `allowed_files`
- test files corresponding to modified modules
- new files within assigned directories (if permitted)

## 5.2 Forbidden

- root-level specification documents
- unrelated modules
- scenario library (unless explicitly assigned)
- scoring logic (unless evaluation task)
- replay system (unless telemetry task)

Violating file boundaries invalidates the task.

---

# 6. Modification Rules

Agents must follow strict modification behavior:

- do not rename files unless instructed
- do not move files between directories
- do not delete files unless explicitly required
- do not introduce new dependencies without approval
- do not refactor unrelated code

Agents are not allowed to “clean up” the repo unless the task says so.

---

# 7. Output Requirements

Each agent task must produce:

## 7.1 Required Outputs

- code changes
- tests (if applicable)
- short summary of changes

## 7.2 Forbidden Outputs

- long essays
- architectural rewrites
- speculative improvements outside scope
- undocumented behavior changes

If an agent writes more explanation than code, something has gone wrong.

---

# 8. Determinism Requirements

All implementations must respect determinism requirements from the architecture. fileciteturn10file3

Agents must ensure:

- no uncontrolled randomness
- seeded randomness where required
- stable outputs for identical inputs

This is non-negotiable for benchmark integrity.

---

# 9. Testing Requirements

Every functional change must include tests.

## Required:

- unit tests for new logic
- deterministic behavior checks
- edge case handling

## Optional (later phases):

- integration tests
- replay validation tests

Implementation phases explicitly require testing gates. fileciteturn10file8

---

# 10. Logging and Replay Compatibility

Agents must ensure:

- all actions produce replay-visible events
- no hidden state mutations
- all scoring signals are traceable

Replay is the audit layer. Breaking it breaks trust. fileciteturn10file7

---

# 11. Anti-Pattern Rules

Agents must avoid the following:

## 11.1 Scope Creep
Adding features not requested.

## 11.2 Silent Behavior Changes
Changing logic without documentation.

## 11.3 Cross-Module Coupling
Linking modules that should remain independent.

## 11.4 Utility Dumping
Putting unrelated logic into `utils/`.

## 11.5 Over-Abstraction
Creating unnecessary layers “just in case.”

---

# 12. Error Handling Expectations

Agents must:

- validate inputs
- handle invalid states gracefully
- avoid crashing the simulation loop
- produce clear error messages

The agent interface spec requires robust handling of malformed actions. fileciteturn10file5

---

# 13. Communication Protocol

Agents should report:

- what was implemented
- what assumptions were made
- any ambiguities encountered

Agents must not invent requirements silently.

---

# 14. Escalation Rules

If a task is unclear, the agent must:

- stop execution
- report ambiguity
- request clarification

Continuing with guesswork is considered failure.

---

# 15. Version Control Behavior

Agents should:

- produce clean diffs
- group related changes
- avoid unrelated formatting changes
- not rewrite large files unnecessarily

---

# 16. Scenario Safety Rules

Agents must not:

- alter canonical scenarios without explicit instruction
- change scoring hooks silently
- introduce exploit paths

Scenarios are benchmark instruments, not playground content. fileciteturn10file10

---

# 17. Evaluation Safety Rules

Agents must not:

- change scoring weights without instruction
- introduce hidden metrics
- alter normalization rules silently

Scoring defines the benchmark’s meaning. fileciteturn10file6

---

# 18. Replay Safety Rules

Agents must not:

- drop replay events
- reorder event logs
- compress away critical data
- hide state transitions

Replay integrity is required for auditability.

---

# 19. Agent Quality Bar

A task is considered successful only if:

- code compiles / runs
- tests pass
- behavior matches specification
- no unintended side effects are introduced

“Looks correct” is not sufficient.

---

# 20. Example Good Task Outcome

- Implements exactly requested feature
- Adds relevant tests
- Leaves unrelated code untouched
- Produces deterministic results
- Matches spec language closely

---

# 21. Example Failure Modes

- modifies multiple subsystems without permission
- adds undocumented features
- breaks determinism
- omits tests
- introduces hidden coupling

---

# 22. Relationship to Other Documents

This playbook enforces:

- Project Charter principles (benchmark-first, modularity) fileciteturn10file0
- System Architecture boundaries fileciteturn10file3
- Repository Structure constraints fileciteturn10file12
- Implementation phase discipline fileciteturn10file8

It is the execution layer for all of them.

---

# 23. Closing Statement

Coding agents are powerful and fast.

They are also extremely good at confidently doing the wrong thing at scale.

This playbook exists so that:

- speed does not destroy structure
- autonomy does not destroy correctness
- and your repo does not become an archaeological site of half-finished ideas

Follow it strictly, or prepare to debug a machine that optimizes for chaos.
