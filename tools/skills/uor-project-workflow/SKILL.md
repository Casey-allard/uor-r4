---
name: uor-project-workflow
description: Execute one build-first UOR-R4 implementation task and deliver its working behavior through the protected pull-request path.
---

# UOR project workflow

Follow the checkout's `AGENTS.md` and
`docs/integration/agent-execution-policy.json`. The project is in
`build_first_pre_alpha` mode until a source-free model artifact loads and
produces prompt-dependent text through the repository CLI or local service on
target local hardware.

## Start

1. Refresh `origin/main`, the live active issue, and its immediate blocker or
   parent when one matters.
2. Read `AGENTS.md` and `docs/integration/current-state.md`.
3. Create one isolated full worktree and implement one product behavior.

Do not begin with a project-wide history reconstruction. Query the knowledge
index, old research, external repositories, or papers only when a concrete code
decision cannot be answered from the current source and active issue. Original
source beats summaries.

## Build the product

Agents may edit, compile, lint, test, train, fit, evaluate, and run repository
CLIs, models, services, and browser flows. Use the smallest command that
directly exercises the behavior changed by the task. A test is useful when it
protects a specific regression; code changes do not automatically require a
new test or a full suite.

Formal proof work, claim/evidence ledgers, knowledge-index maintenance,
experiment freezes, independent review, replay packages, receipt chains,
publication work, NEMESIS/W33 mapping, duplicate roadmap/status updates, source
audits, and broad release QA are deferred until the working-alpha condition.
Only an explicit owner instruction can activate one earlier.

Automatic retries are zero. When a command fails, inspect the output, make one
direct source or input correction, and rerun once only when that correction
plausibly addresses the failure. Do not create supervisors, watchdogs, receipt
harnesses, workspace capsules, or environment-probe campaigns.

A command expected to exceed 15 minutes, create more than 10 GiB, or incur an
external cost needs explicit owner or active-issue authorization, a hard limit,
and a stop condition.

## Preserve and deliver

Preserve unrelated changes, user material, unique artifacts, and prior negative
results. Keep mathematical proof, measured behavior, and unverified hypotheses
distinct without manufacturing an evidence dossier.

Deliver through a protected pull request. A routine completion report contains
the working behavior, the command and observed result, the remaining limitation,
the closure state, and one next implementation action. Stop after the one task.
