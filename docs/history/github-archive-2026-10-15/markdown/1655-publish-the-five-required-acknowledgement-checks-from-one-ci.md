# #1655 — Publish the five required acknowledgement checks from one CI job

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T19:02:12Z
- **Updated:** 2026-10-09T13:09:07Z
- **Closed:** 2026-10-03T22:11:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1655
- **Merged:** 2026-10-03T22:11:35Z
- **Merge commit:** e9364c9850edf34f0c240477438e6f1a352d6d3e
- **Base / Head:** main / UOR-Foundation:codex/no-required-checks
- **Reviews / inline review comments:** 2 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Owner-approved, 3 October 2026.**

**Problem.** Main's ruleset requires five status names that are compatibility acknowledgements. They run no formatting, Clippy, tests, audit, fuzzing, WASM build or model evaluation.
- Each name was its own job, so every PR push and every merge-queue entry needed five slots in the organization's shared GitHub runner pool.
- When other repositories hold that pool, merges stall. #1644 was removed from the queue with `checks_timed_out`.

**Change** (`.github/workflows/ci.yml`)
- One job, `required-transport` with `checks: write`, replaces the five transport jobs.
- It publishes the five names as completed check runs on the commit the ruleset evaluates: the PR head for `pull_request`, or the merge-group head for `merge_group`.
- The names are exactly the ruleset's contexts:
  - `fmt / clippy / tests / no_std / κ`
  - `cargo audit`
  - `fuzz smoke (nightly libfuzzer)`
  - `wasm-pack build (dashboard facade, deploy.yml parity)`
  - `Gate C trend alarm`
- Each check run's summary says that no test ran.
- One runner slot instead of five; same no-op meaning as before.
- Real validation remains the exact-head evidence posted on each PR.

AGENTS.md, STATUS.md and the uor-r4-api README now describe this. Removing the names from ruleset 19597522 altogether still needs an organization admin.

**Verification.** This PR's own `pull_request` run uses the changed workflow, so the five names posted on its head prove the transport works. The merge-group run proves it again on the queue path before merge.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### review 5402310594 — 2026-10-03T19:02:42Z — Casey-allard — COMMENTED

**Exact-head review at `c20aef014fb6dfb8efd5c7771924af76cc3c5c12` (Claude lab; self-review under the owner's 2026-09-30 rule).** The change is documentation only. AGENTS.md, STATUS.md, the uor-r4-api README and the ci.yml header comment now match the owner's 3 October decision: the ruleset keeps the merge queue and drops the five no-op required contexts. No Rust or workflow logic changed. `python3 scripts/check_claim_wording.py` passed, and ci.yml parses. Historical evidence JSON is untouched. **APPROVE**; queueing with `--match-head-commit`.

### comment 5972570545 — 2026-10-03T19:10:14Z — Casey-allard

Removed from the merge queue: the ruleset change needs an org admin, and the requesting account has write access only. This PR is re-queued at its exact head once an admin unchecks *Require status checks to pass* in ruleset `19597522`, so the docs never describe a setting that isn't in effect.

### review 5402991598 — 2026-10-03T22:10:43Z — Casey-allard — COMMENTED

**Exact-head review at `5f39b6eacb0cd95c24e5caeb86f331036ac62885` (Claude lab; self-review under the owner's 2026-09-30 rule; this change is owner-approved).**

**What changed**
- `ci.yml` replaces the five transport jobs with one `required-transport` job (`checks: write`).
- The YAML parses, and the job publishes exactly the five ruleset contexts. Each summary states that no test ran.
- Docs updated; claim-wording passes.

**Proof on this PR's own `pull_request` run:** all five names were posted on head `5f39b6eacb0cd95c24e5caeb86f331036ac62885` by `github-actions`, `completed/success`, from the single job.

The merge-group run exercises the same path on the queue. **APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5972570545, 5402310594, 5402991598_

