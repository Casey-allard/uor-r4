# MUDBench Agent Runtime Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the runtime scaffold that wraps an autonomous coding or benchmark agent so it can operate safely, iteratively, and reproducibly inside the MUDBench project.

---

# 1. Why This Exists

The MUDBench specification layer defines:

- what the system is
- how it is structured
- how agents are evaluated
- how coding agents are constrained

What it does **not** define by itself is the runtime mechanism that turns a model into an operational agent.

This document defines that missing layer.

The runtime is the scaffold that allows an agent to:

- read project state
- interpret tasks
- inspect files
- make bounded changes
- run validations
- observe results
- iterate safely

Without the runtime, a model is just text generation.
With the runtime, it becomes a constrained software agent.

---

# 2. Relationship to Existing Specifications

This runtime must respect the existing MUDBench design stack.

It must operate consistently with:

- Project Charter
- System Architecture
- Repository Structure
- Coding Agent Playbook
- Task Template Library
- Validation and Guardrails
- CI and Automation Pipeline

The runtime does not replace those documents.
It enforces and operationalizes them.

---

# 3. Core Runtime Principles

The agent runtime must be:

- deterministic where possible
- task-scoped
- tool-mediated
- auditable
- replayable
- interruptible
- validation-first

The runtime must never grant broader power than the assigned task requires.

---

# 4. Runtime Role

The runtime is responsible for converting a static model into an interactive agent.

It provides:

- task intake
- context assembly
- tool access
- execution loop
- validation loop
- artifact recording
- failure handling

The runtime is not the benchmark itself.
It is the operational shell around the agent.

---

# 5. Canonical Agent Loop

The runtime must implement a bounded iterative loop:

1. receive task
2. load allowed context
3. inspect relevant files
4. plan local action
5. perform tool-mediated action
6. run validation
7. inspect results
8. either finish or continue within task limits

Canonical loop:

```text
task → context → inspect → act → validate → observe → iterate → finalize
```

This loop must remain explicit and logged.

---

# 6. Runtime Modes

The runtime should support at least three modes.

## 6.1 Build Mode

Purpose:
- implement repository features
- edit code
- create tests
- run local validation

Typical use:
- Codex-style autonomous repo work

## 6.2 Evaluation Mode

Purpose:
- run agents inside MUDBench scenarios
- collect scorecards, telemetry, and replays

Typical use:
- benchmark execution

## 6.3 Analysis Mode

Purpose:
- inspect existing runs
- summarize registry entries
- diagnose failures
- compare artifacts

Typical use:
- debugging and research analysis

Each mode must have different tool permissions.

---

# 7. Task Intake

The runtime must accept tasks in the canonical structured format defined by the task template library.

Minimum required task fields:

- task_id
- role
- objective
- scope
- allowed_files
- inputs
- constraints
- deliverables
- output_format

If any required field is missing, the runtime must reject the task before execution.

---

# 8. Context Assembly

Before execution, the runtime must assemble bounded context.

Context may include:

- allowed specification documents
- assigned code files
- relevant test files
- prior task state (if permitted)
- configuration files within scope

The runtime must not expose unrelated files by default.

Context assembly rules:

- include only what is needed
- prefer exact paths over repository-wide scans
- preserve task boundaries
- log the context set used for execution

---

# 9. Tool Access Model

All agent actions must be mediated through tools.
The model must not be treated as if it has magical direct control over the environment.

Tool categories may include:

## 9.1 Read Tools
- read file
- list directory
- inspect task state
- inspect registry or replay metadata

## 9.2 Write Tools
- edit allowed files
- create new files in allowed directories
- update tests within scope

## 9.3 Execution Tools
- run unit tests
- run linters
- run local benchmark scenario
- run deterministic validation

## 9.4 Analysis Tools
- inspect diffs
- compare run outputs
- summarize replay metadata
- inspect scorecards

Every tool call must be logged.

---

# 10. Permission Boundaries

The runtime must enforce hard boundaries.

## Allowed
- files explicitly listed in task scope
- tests corresponding to edited modules
- local outputs defined by task contract

## Forbidden
- root specs unless explicitly assigned
- unrelated subsystem directories
- secret/config files outside scope
- arbitrary shell exploration
- deletion outside explicit task authorization

Permission violations must stop execution immediately.

---

# 11. Memory and State Handling

The runtime must manage agent working state explicitly.

Runtime state may include:

- current task
- files inspected
- edits attempted
- validations run
- unresolved errors
- iteration count

This state must be:

- persisted per task where possible
- reset between unrelated tasks
- logged for auditability

The runtime must never rely on undocumented hidden memory.

---

# 12. Planning Behavior

The runtime may allow internal planning, but plans must remain bounded.

A runtime plan should include:

- files to inspect
- changes to make
- validations to run
- success criteria

The runtime must prevent endless planning loops by enforcing:

- iteration limits
- tool call limits
- timeout limits

Planning is allowed.
Unbounded wandering is not.

---

# 13. Execution Semantics

The runtime must treat model output as a proposal, not a trusted operation.

Execution process:

1. model proposes next action
2. runtime checks whether the action is permitted
3. permitted action is executed through tool layer
4. result is returned to agent
5. state is updated
6. loop continues or stops

This separation is mandatory for safety.

---

# 14. Validation Loop

The runtime must integrate validation directly into the agent loop.

Validation triggers may include:

- after every file edit
- after every logical task milestone
- before finalization
- before merge or artifact publication

Validation types:

- syntax validation
- task-boundary validation
- unit tests
- determinism checks
- replay checks
- score recomputation checks

The runtime must refuse to finalize a task that has not passed required validation gates.

---

# 15. Failure Handling

The runtime must distinguish between failure types.

## 15.1 Task Failure
Examples:
- ambiguous task
- missing files
- invalid scope

Response:
- reject task before work begins

## 15.2 Execution Failure
Examples:
- test failure
- syntax error
- validation mismatch

Response:
- log failure
- permit bounded retry if allowed
- otherwise finalize as failed

## 15.3 Permission Failure
Examples:
- attempted write outside scope
- forbidden command
- unauthorized file access

Response:
- immediate halt
- mark task invalid
- escalate to controller or human

---

# 16. Retry Policy

Retries must be bounded and policy-driven.

Recommended rules:

- retry only for correctable execution failures
- never retry permission violations automatically
- limit retries per task
- record each retry attempt explicitly

Unlimited retries produce noise and hide systemic failures.

---

# 17. Finalization Rules

A task may only finalize successfully if:

- all required deliverables exist
- file changes remain within scope
- required tests pass
- required validations pass
- output format matches task contract

Final task record should include:

- task_id
- files changed
- tests run
- validation outcomes
- summary
- status
- artifact references if any

---

# 18. Runtime Logging

The runtime must log:

- task intake
- context set
- tool calls
- file changes
- validations run
- failures
- final outcome

Logs must be sufficient to reconstruct what the agent actually did.

If the runtime cannot explain an agent action later, the runtime is incomplete.

---

# 19. Replayability of Runtime Sessions

Agent runtime sessions should themselves be replayable at the operation level.

A runtime replay should reconstruct:

- input task
- context provided
- sequence of tool invocations
- validation results
- final outputs

This is separate from benchmark replay, but conceptually parallel.

MUDBench already treats replay as a trust mechanism for benchmark behavior.
The runtime should use the same philosophy for autonomous build behavior.

---

# 20. Integration with CI

The runtime must integrate with CI rather than bypass it.

Required behavior:

- local runtime validation before submission
- CI re-validation before acceptance
- artifact handoff for failed or successful runs
- explicit linkage between runtime session and CI job where possible

The runtime is a pre-merge executor, not an alternative truth source.

---

# 21. Integration with Experiment Registry

When used in evaluation or analysis mode, the runtime must register:

- experiment identifiers
- run identifiers
- artifact paths
- relevant hashes
- version metadata

This ensures that autonomous execution remains connected to provenance tracking.

---

# 22. Human Oversight Hooks

The runtime must support human checkpoints.

Human approval may be required for:

- specification edits
- scoring changes
- scenario-library modifications
- release actions
- merge to protected branches

Autonomy must be interruptible.

A human lead must be able to:

- stop execution
- inspect current state
- approve continuation
- force failure or reset

---

# 23. Multi-Agent Runtime Coordination

Future versions may support multiple coordinated coding agents.

If enabled, the runtime must define:

- role separation
- task isolation
- shared state rules
- conflict resolution
- merge ordering

Until such coordination is explicitly implemented, the runtime should assume one active agent per task scope.

---

# 24. Security Constraints

The runtime must prohibit:

- arbitrary internet access unless explicitly enabled
- arbitrary package installation
- access to secrets outside task scope
- uncontrolled shell operations
- hidden background execution
- undeclared tool use

Security is part of reproducibility.
If agents can do undocumented things, results lose meaning.

---

# 25. Recommended Runtime Outputs

For build-mode tasks, the runtime should emit:

- patch or changed files
- validation summary
- concise implementation note

For evaluation-mode tasks, the runtime should emit:

- run summary
- scorecard path
- replay path
- telemetry summary

For analysis-mode tasks, the runtime should emit:

- query result
- artifact references
- short reasoning summary

Outputs must remain structured and machine-consumable where possible.

---

# 26. Minimal Runtime State Machine

A minimal state machine for the runtime:

```text
IDLE
→ TASK_RECEIVED
→ CONTEXT_READY
→ EXECUTING
→ VALIDATING
→ RETRYING (optional)
→ COMPLETED | FAILED | HALTED
```

State transitions must be logged.

---

# 27. Reference Runtime Deployment Model

A practical initial deployment may use:

- one controller process
- one active model session per task
- filesystem sandbox
- local test runner
- structured task JSON
- artifact logger

This is enough to support autonomous repo work without overbuilding the runtime.

Start simple.
Scale only after the control loop is stable.

---

# 28. Anti-Patterns

The runtime must avoid:

- giving the model unrestricted repo access
- allowing edits before task validation
- skipping validation because output “looks fine”
- letting the model define its own task boundaries
- relying on conversation memory instead of explicit state
- allowing hidden side effects outside logs

These are not harmless shortcuts.
They are system-corrupting behaviors.

---

# 29. Relationship to Future Work

This runtime spec can later support:

- Codex-driven autonomous implementation
- multi-agent software construction
- autonomous scenario generation
- regression triage agents
- benchmark-analysis agents
- controlled self-improvement loops

But none of that should be enabled until the minimal runtime works predictably and auditably.

---

# 30. Closing Statement

The runtime is the missing bridge between specification and autonomy.

The specs tell the agent what reality should look like.
The runtime determines whether the agent can interact with that reality safely.

If the runtime is weak, autonomy becomes chaos.
If the runtime is strong, autonomy becomes engineering.

That is the entire game.
