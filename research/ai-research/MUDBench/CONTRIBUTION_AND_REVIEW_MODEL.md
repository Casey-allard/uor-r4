# MUDBench Contribution and Review Model

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how contributions (human or agent) are proposed, reviewed, validated, and merged into the MUDBench repository without compromising benchmark integrity.

---

# 1. Why This Exists

At this point, your system is:

- modular
- agent-operable
- reproducible
- heavily constrained

Which means the next threat is obvious:

**other contributors.**

This document ensures that:
- contributions improve the system
- review is structured and enforceable
- no one "accidentally" breaks benchmark integrity

---

# 2. Core Principles

All contributions must be:

- bounded in scope
- reproducible
- aligned with specifications
- validated through CI
- reviewable through artifacts (not opinions)

If a contribution cannot be objectively evaluated, it does not belong in the system.

---

# 3. Contribution Types

## 3.1 Code Contributions
- feature implementation
- bug fixes
- performance improvements

## 3.2 Specification Changes
- updates to core docs
- architecture adjustments
- scoring changes

## 3.3 Scenario Contributions
- new benchmark scenarios
- scenario refinements

## 3.4 Test and Validation Contributions
- new tests
- validation improvements

---

# 4. Contribution Workflow

All contributions follow this pipeline:

1. Task Definition
2. Implementation
3. Local Validation
4. Pull Request Submission
5. Automated CI Validation
6. Human Review
7. Merge or Reject

Skipping steps is how systems decay.

---

# 5. Pull Request Requirements

Every PR must include:

- clear objective
- bounded scope
- linked task (if applicable)
- affected files list
- test coverage
- summary of changes

Optional (but encouraged):
- replay artifacts
- benchmark impact notes

---

# 6. Automated CI Gate

Before human review, CI must pass:

- linting
- unit tests
- determinism checks
- simulation validation
- replay validation
- scoring validation

Failure = automatic rejection.

No discussion required.

---

# 7. Human Review Responsibilities

Human reviewers must verify:

- alignment with specifications
- no hidden behavior changes
- correct scope boundaries
- clarity and maintainability

Review is not:
- subjective preference
- style policing
- speculative redesign

---

# 8. Review Categories

## 8.1 Standard Review
For normal code contributions.

## 8.2 Critical Review
Required for:
- scoring changes
- scenario changes
- replay system changes
- architecture changes

Requires:
- deeper inspection
- explicit approval

---

# 9. Merge Rules

A PR may only be merged if:

- CI passes
- required reviewers approve
- no unresolved comments remain

No exceptions.

---

# 10. Rejection Criteria

Reject contributions that:

- violate determinism
- break replay integrity
- introduce hidden state
- exceed scope
- lack tests
- modify restricted files

Partial correctness is still failure.

---

# 11. Specification Change Protocol

Spec changes are dangerous.

They require:

- explicit version update
- justification
- backward compatibility consideration
- impact analysis

Specs define reality in this system.

Changing them is not casual.

---

# 12. Scenario Governance

Scenario contributions must:

- align with scenario spec
- avoid trivial exploits
- map cleanly to scoring signals
- pass deterministic validation

Scenarios are measurement tools.

Treat them like instruments, not content.

---

# 13. Agent-Generated Contributions

Agent contributions must:

- follow task templates
- respect allowed_files
- produce deterministic outputs

Extra scrutiny required for:

- large diffs
- cross-module changes
- unclear logic

Agents are fast.
They are also very good at being confidently wrong.

---

# 14. Version Control Discipline

Rules:

- no force pushes to main
- no direct commits to protected branches
- meaningful commit messages required

---

# 15. Branch Strategy

Recommended:

- main → stable
- dev → integration
- feature/<name> → contributions

---

# 16. Auditability Requirements

Every merged change must be:

- traceable to a PR
- reproducible via CI
- explainable via diff

If you cannot explain a change later, you should not merge it now.

---

# 17. Conflict Resolution

When disagreements occur:

- defer to specifications
- prioritize determinism
- escalate to architecture authority if needed

Not everything is a democracy.

---

# 18. Contribution Anti-Patterns

Do not allow:

- "quick fixes" without tests
- silent refactors
- cross-module rewrites
- undocumented changes
- speculative optimizations

These are how systems rot.

---

# 19. Continuous Improvement Loop

After merge:

- monitor CI stability
- monitor replay consistency
- track regression signals

Bad merges must be reverted quickly.

---

# 20. Closing Statement

You are building a system where:

- agents write code
- code defines evaluation
- evaluation defines intelligence

Letting uncontrolled contributions into that system is not collaboration.

It is entropy.

This model exists to make sure every change is:

- intentional
- validated
- and worth the cost of its existence
