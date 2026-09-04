---
name: uor-project-workflow
description: Execute one build-first UOR-R4 implementation task and deliver its working behavior through the protected pull-request path.
---

# UOR project workflow

Follow the checkout's `AGENTS.md` and
`docs/integration/agent-execution-policy.json`. The project is in
`build_first_architectural_alpha` mode. Follow the ordered stages in
`docs/integration/project-track.md`; do not substitute an older issue or proof
sequence.

## Start

1. Refresh `origin/main`, the live active issue, and its immediate blocker or
   parent when one matters.
2. Read `AGENTS.md`, `docs/integration/current-state.md`, and
   `docs/integration/project-track.md`.
3. Create one isolated full worktree and implement one product behavior.

Do not begin with a project-wide history reconstruction. Query the knowledge
index or inspect SpiralCore, HELM, W33, NEMESIS, UOR, H4/zeta, other external
repositories, or papers when they can answer a concrete active design question.
Treat them as donor reservoirs and inspect original source before transferring a
mechanism or claim.

## Build the product

Agents may edit, compile, lint, test, train, fit, evaluate, and run repository
CLIs, models, services, and browser flows. Use the smallest command that
directly exercises the behavior changed by the task. A test is useful when it
protects a specific regression; code changes do not automatically require a
new test or a full suite.

Bounded iteration on open development data is allowed; keep final held-out
evaluation after design selection. Proportionate independent review is useful
for novel causal/state code and when the owner requests it. Broad formal proof,
claim-ledger reconciliation, routine knowledge-index maintenance, replay and
receipt packages, publication, programme-wide research mapping, duplicate
plans, and broad release QA wait for the release candidate unless a concrete
implementation decision or owner activates one earlier.

Automatic retries are zero. When a command fails, inspect the output, make one
direct source or input correction, and rerun once only when that correction
plausibly addresses the failure. Do not create supervisors, watchdogs, receipt
harnesses, workspace capsules, or environment-probe campaigns.

A command expected to exceed 15 minutes, create more than 10 GiB, or incur an
external cost needs explicit owner or active-issue authorization, a hard limit,
and a stop condition.

## Preserve and deliver

Preserve unrelated changes, user material, unique artifacts, and prior negative
results. A negative binds its exact artifact, population, operator, controls,
budget, and decision; a materially versioned successor may re-enter with a
named rationale. `UNAVAILABLE` is not model evidence. Keep mathematical proof,
measured behavior, and unverified hypotheses distinct without manufacturing an
evidence dossier.

Deliver through a protected pull request. A routine completion report contains
the working behavior, the command and observed result, the remaining limitation,
the closure state, and one next implementation action. Stop after the one task.
