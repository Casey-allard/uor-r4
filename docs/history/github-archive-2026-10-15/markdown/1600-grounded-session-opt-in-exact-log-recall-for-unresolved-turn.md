# #1600 — Grounded session: opt-in exact log recall for unresolved turns (session MQAR 42 → 79/109)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T04:20:07Z
- **Updated:** 2026-10-09T13:08:48Z
- **Closed:** 2026-10-02T05:23:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1600
- **Merged:** 2026-10-02T05:23:25Z
- **Merge commit:** 4caf1e54e2a5bd7fc33a598bd8b183d360412b9c
- **Base / Head:** main / UOR-Foundation:codex/session-log-recall
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552 (D19), #1512 (prime router).

`GroundedSession::with_log_recall(LogRecall)` supplies `Memory: <value>.` on turns the compiler leaves unresolved, when reading is enabled. The value comes from a deterministic provider over the prior user turns. It is recorded as `RecallDisposition::LogValue` and is **off by default**.

Claimed on #1552 before editing `stack_grounded_session.rs`. The change is one field, one builder, and the prior-turn slice threaded through `emitter_suffix` so rebuilt and replayed history reproduces each line.

- `milestone_world_v2::sieve_value_text`: the exact prime-atom sieve over plain texts. `sieve_value` now delegates to it, with identical behaviour.
- `m-world session log_recall=off|sieve`, recorded in the report.

**Checks.** New test `log_recall_is_off_by_default_and_rebuilds_from_prior_turns_only`; the 19 `stack_grounded_session` + sieve tests pass. Evidence branch: `codex/ci/session-log-recall`.

**Measured** (#1552): with emit-3 × `unless_query`, MQAR 42 → **79/109**; relation 73/83, responsive 546/752 and instruction 23/98 unchanged.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5946088003 — 2026-10-02T05:22:32Z — Casey-allard

**Exact-head review: `6dfae5021f126f2d4a1d500c225b8d06fa2f7634`** (self-review, under the owner rule of 30 September on #820).

**Scope checked.**
- `log_recall` defaults to `None`, so sessions are unchanged unless enabled.
- The provider sees only `prior` turns at every `emitter_suffix` call site: the live turn, `emitter_input`, `closed_history` and the load replay. That makes rebuilt lines reproducible, as the new test checks.
- Log recall applies only to `MemoryEffect::Unresolved` with reading enabled; store reads keep their own recall.
- `sieve_value` behaviour is unchanged: it delegates to `sieve_value_text`.
- The session binds the protocol version recorded in the emitter's training record, defaulting to 1 when absent. `/1` sessions reproduce earlier results (emit-3 × `unless_query`: 44/52).

**Evidence at this head:** run on `codex/ci/session-log-recall` (`752c5fad`): fmt and the check of all targets succeeded; `uor-r4-training` lib 517 passed, 0 failed; examples pass. The PR merges cleanly with the current main.

**Measured** (#1552): session MQAR 42 → 79/109 (`/1`); best `/2` configuration total 760 (copy 5/33, MQAR 91/109).

Verdict: approve for merge at this head.

### comment 5946143556 — 2026-10-02T05:28:34Z — Casey-allard

**Merged** as `4caf1e54` through the queue at head `6dfae502`; main's tree equals the PR head's tree.


---
_Comment/review ids in this file: 5946088003, 5946143556_

