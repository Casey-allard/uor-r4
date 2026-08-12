# MUDBench Validation and Guardrails

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how MUDBench enforces correctness, determinism, and structural integrity across all agent-driven development and benchmark execution.

---

# 1. Why This Exists

You now have:
- specifications
- architecture
- agent playbook
- task templates

That’s enough power to build the system.

It’s also enough power to break it in subtle, irreversible ways.

This document defines:
- how work is validated
- how incorrect outputs are rejected
- how the system protects itself from agents doing “technically valid but actually destructive” work

Without guardrails, validation becomes subjective.
Subjectivity destroys reproducibility.

---

# 2. Core Principle

Everything must be:

- deterministic
- reproducible
- auditable
- bounded

If it cannot be verified, it does not count.

---

# 3. Validation Layers

Validation is not a single step.

It is a layered system:

1. Task Validation
2. Code Validation
3. Simulation Validation
4. Replay Validation
5. Benchmark Validation

Each layer must pass before the next is trusted.

---

# 4. Task-Level Validation

Before execution:

- task must follow canonical schema
- scope must be explicit
- allowed_files must be bounded
- constraints must be defined

Reject task if:

- objective is vague
- files are unspecified
- constraints are missing

Bad input → bad output.
Do not proceed.

---

# 5. Code-Level Validation

After agent output:

## Required checks:

- code compiles / runs
- no unauthorized file modifications
- no cross-module violations
- no hidden dependencies introduced

## Forbidden patterns:

- editing root spec files
- modifying unrelated subsystems
- silent behavior changes

If any violation occurs → reject output.

---

# 6. Determinism Validation

All systems must produce identical results given identical inputs.

## Required:

- seeded randomness only
- stable ordering of operations
- no time-dependent logic
- no hidden global state

## Validation method:

- run identical simulation twice
- compare outputs byte-for-byte

Mismatch = failure

---

# 7. Simulation Validation

Each run must satisfy:

- valid world initialization
- valid agent actions
- no invalid state transitions
- no crashes in simulation loop

Invalid transitions must be caught and logged.

Silent failure is unacceptable.

---

# 8. Replay Validation

Replay is the audit layer.

Every run must:

- produce a complete event log
- allow full reconstruction of state
- preserve event ordering

Validation checks:

- replay reproduces identical outcome
- no missing events
- no hidden state

If replay diverges → entire run invalid.

---

# 9. Scenario Validation

Scenarios must be validated before use.

Checks:

- deterministic with seed
- no trivial exploit paths
- scoring hooks fire correctly
- termination conditions reachable

Scenarios are measurement instruments.

A broken scenario invalidates all results derived from it.

---

# 10. Scoring Validation

Evaluation must be:

- transparent
- replay-auditable
- deterministic

Checks:

- score recomputation from replay matches original
- no hidden metrics
- no external dependencies

If scoring cannot be reconstructed, it is not valid.

---

# 11. File Boundary Enforcement

Agents must not exceed assigned scope.

Validation must ensure:

- only allowed_files were modified
- no directory restructuring occurred
- no implicit file creation outside scope

Violation → reject task output.

---

# 12. Anti-Corruption Guardrails

Prevent slow degradation of system integrity.

## Disallow:

- convenience shortcuts in scoring
- silent coupling between modules
- logic hidden in utils/
- bypassing validation layers

These are the beginnings of benchmark collapse.

---

# 13. Test Gate Requirements

All code must pass:

- unit tests
- determinism tests
- edge case tests

Future phases:

- integration tests
- replay verification tests

No passing tests → no merge.

---

# 14. Output Format Enforcement

Agents must respect required output formats.

Examples:

- STRICT_JSON_ONLY
- code only
- code + summary

Deviation indicates:

- misunderstanding
- hallucination
- non-compliance

Reject output.

---

# 15. Failure Handling

When validation fails:

- reject output
- log failure reason
- do not partially accept results
- require task revision

Partial correctness is not acceptable in benchmark systems.

---

# 16. Version Integrity

All components must be versioned:

- tasks
- scenarios
- metrics

Validation must ensure:

- version identifiers are present
- changes are explicit
- no silent mutation

---

# 17. Continuous Validation Pipeline

Validation should be automated.

Pipeline:

1. task check
2. agent execution
3. code validation
4. test execution
5. simulation run
6. replay validation
7. scoring validation

Failure at any stage stops progression.

---

# 18. Human Oversight Layer

Even with automation:

- architecture changes require human review
- scoring changes require explicit approval
- scenario changes require validation audit

Agents do not govern the benchmark.

---

# 19. Common Failure Modes

- deterministic drift (subtle randomness introduced)
- replay mismatch
- hidden state leakage
- scoring inconsistency
- cross-module coupling

These failures are often silent at first.

That is why guardrails exist.

---

# 20. Closing Statement

You are building a system where:

- agents write code
- code defines evaluation
- evaluation defines intelligence

If validation is weak, the entire system becomes meaningless.

Guardrails are not overhead.

They are the only thing preventing your benchmark from quietly lying to you.
