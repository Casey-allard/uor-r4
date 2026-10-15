# #1108 — Enforce deterministic source-only execution for automated agents

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-04T03:54:35Z
- **Updated:** 2026-09-04T03:59:16Z
- **Closed:** 2026-09-04T03:58:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1108
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The automated development workflow can currently waste time and tokens by reacting to an execution-environment failure with partial workspace copies, diagnostic probes, supervisor wrappers, and retries. A partial copy can also omit declared build targets and manufacture a failure unrelated to the product decision.

Deliver a repository-wide deterministic source-only agent policy:

- require a complete Git worktree from refreshed `origin/main`;
- prohibit sparse/pruned/hand-copied workspaces;
- prohibit agent-run or agent-dispatched builds, tests, probes, model work, QA, supervisors, and automatic retries;
- keep product/release QA owner-operated through explicit `workflow_dispatch` only;
- allow one source correction after a concrete owner-run remote failure, then park;
- suppress per-build/test/probe/poll narration;
- enforce the policy with a standard-library static guard in an existing required merge status.

Acceptance:

- machine-readable versioned policy and human rationale are tracked;
- `AGENTS.md` and `CONTRIBUTING.md` bind automated work to it;
- pull-request and merge-group automation cannot add executable QA without failing the guard;
- the existing five ruleset status names and protected merge queue remain intact;
- no project build, test, model, browser, or operating-system probe is run for this change.


## Comments (1)

### comment 5535448446 — 2026-09-04T03:59:06Z — Casey-allard

Delivered through protected PR #1109 and the required merge queue.

- reviewed head: `84f607741d4ef59c8bbb21d94d75d3ad8dfd33c8`
- merge commit: `aff4acc41f07b866c0f4152440e9028d3cec8088`
- machine contract: `docs/integration/agent-execution-policy.json`
- binding instructions: `AGENTS.md` and `CONTRIBUTING.md`
- enforcement: `scripts/check_agent_execution_policy.py` runs in the existing required core status for pull requests and merge groups; all executable product/release QA jobs remain restricted to owner-operated `workflow_dispatch`
- static result: `agent execution policy: OK (static text only; no project code executed)` on the reviewed merge preview
- project execution: `NOT_RUN_BY_POLICY`; no Cargo/Rust build, test, probe, model run, browser/service check, OS diagnostic, supervisor, or retry loop ran

This closes the automated-workflow failure mode: complete Git worktrees only, zero environment probes, zero automatic retries, no agent-dispatched QA, one source correction after a concrete owner-run remote failure, then park. It does not establish determinism of research algorithms, third-party tools, operating systems, or model outputs.

Next action: begin the next eligible project issue from refreshed `origin/main` under `deterministic_source_only` mode and judge it from source plus owner-supplied remote evidence only.



---
_Comment/review ids in this file: 5535448446_

