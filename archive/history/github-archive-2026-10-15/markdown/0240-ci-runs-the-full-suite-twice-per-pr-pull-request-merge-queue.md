# #240 — CI runs the full suite twice per PR (pull_request + merge queue); split jobs by trigger context and drop the strict up-to-date rule

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:23:28Z
- **Updated:** 2026-07-28T03:48:47Z
- **Closed:** 2026-07-28T03:47:54Z
- **Labels:** enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/240
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Problem

Every PR pays for the full CI suite twice: once in `pull_request` context, then again in the merge queue (`merge_group`) on the speculative merge. The queue run is the valuable one — it tests the PR merged with current `main` plus anything ahead in the queue, which per-PR CI cannot do. The `pull_request` run of the heavy jobs adds latency and Actions cost without adding a verdict anyone acts on: their binding result is the queue's.

Additionally, the ruleset ("main: required checks", id 19597522) still carries the **strict up-to-date branch** requirement from before the merge queue was enabled (AGENTS.md documents the pre-queue setup). With the queue active this is fully redundant — the speculative merge covers everything branch-freshness covered — and it forces rebase→full-rerun cycles on top of the queue's rerun.

### Proposal

Split `ci.yml` jobs by trigger context, keeping every required-check name reporting in both contexts (stub jobs report success where the real work is deferred — the queue cannot merge otherwise):

| Job | `pull_request` | `merge_group` | Rationale |
|---|---|---|---|
| `fmt / clippy / tests / no_std / κ` | **real** | **real** | Core gate; reviewers need real test signal, queue needs it against the speculative merge. The one intentional duplicate. |
| `cargo audit` | **real** | stub | Advisory-DB lookup on the lockfile — merge-content-independent; re-running minutes later in the queue adds nothing. Stays real on `main` pushes. |
| `fuzz smoke (nightly libfuzzer)` | stub | **real** | Heavy; low review signal; binding verdict belongs to the queue. |
| `wasm-pack build` | stub | **real** | Same. |
| `Gate C trend alarm` | stub | **real** | Trend vs pinned record is most meaningful at merge time. |

`Formal Verification (Kani)` is unchanged — it is not in the required-check set and already runs once per PR (no `merge_group` trigger).

Nothing is dropped: every check still runs for real at least once per merge, on the context where its verdict is binding; `push: main` behavior is unchanged.

Separately: remove the strict up-to-date requirement from ruleset 19597522 (admin action; the workflow change does not depend on it, but the two together remove both halves of the duplication).

### Expected effect

Per-PR runner time drops by roughly the fuzz+wasm+GateC cost (~8 min of runners) and queue time by the audit cost; forced rebase re-runs disappear with the ruleset change. At the current PR cadence this is hours of Actions time per day.

### Origin

Discussion during #232/#238 work; the #239 fuzz failure (pin vs nightly-default assumption) is an instance of the same class of context-coupling this makes explicit.


## Comments (1)

### comment 5099708357 — 2026-07-28T03:48:47Z — Casey-allard

**DoD evidence** (closed by PR #241, merge commit `02781eb`):

- wasm-pack / Gate C trend / fuzz smoke now run for real only in `merge_group` (and `push: main` / dispatch), reporting stub success in `pull_request` context (~4s each vs ~2.5-3m real runs per PR).
- `cargo audit` inverted: real in `pull_request` + `main` pushes, stub in the queue (advisory-DB lookup is merge-content-independent).
- Core `fmt / clippy / tests / no_std / κ` unchanged — real in both contexts by design (reviewer signal + queue verification of the speculative merge).
- Every required-check name reports in every gating context; queue mechanics unchanged; nothing stopped running for real at least once per merge. Kani untouched.
- PR #241 itself demonstrated the split: stubs green in seconds on the PR, real suite once in the queue.

Remaining half of this issue (admin action, not automatable from this session): remove the strict "require branches to be up to date" from ruleset 19597522 — with the queue active it only forces redundant rebase→rerun cycles.

Lesson recorded for the next person: plain-scalar `run:` values in YAML treat ` #` as a comment start — the first push shipped silently-truncated echo commands (fixed in `fbf3876` by quoting at the YAML level).



---
_Comment/review ids in this file: 5099708357_

