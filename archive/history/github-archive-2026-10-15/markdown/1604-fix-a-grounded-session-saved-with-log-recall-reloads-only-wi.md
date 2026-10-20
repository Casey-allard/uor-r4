# #1604 — Fix: a grounded session saved with log recall reloads only with that provider

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T06:15:07Z
- **Updated:** 2026-10-09T13:08:44Z
- **Closed:** 2026-10-02T06:46:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1604
- **Merged:** 2026-10-02T06:46:58Z
- **Merge commit:** 69c068ba9df71071b76dbd6603cfb18b35f63b85
- **Base / Head:** main / UOR-Foundation:codex/log-recall-reload-main
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552 (D19 durability). This fixes a defect in #1600.

**Defect (measured).** Saving and reloading the best configuration (`log_recall=sieve`) gave `equal=false` for most reloaded conversations. With log recall off, reload is `equal=true` 8/8. A reload restored the turns but not the provider, so replayed memory lines differed.

**Fix.**
- `with_log_recall(name, recall)` names the provider, and the session record stores the name (an optional field, absent for sessions without one).
- `load_with_log_recall` attaches the provider before replay. It refuses a missing, extra or differently named one; `load` is `load_with_log_recall(.., None)`.
- `m-world session` reloads with its configured provider.

**Measured after the fix:** best configuration (emit-4p2a, `unless_query`, `log_recall=sieve`, cap 64), reload **20/20 equal**, scores unchanged (copy 12/33, instruction 25/98, MQAR 108/109, relation 74/83, responsive 572/752).

**Checks.** The extended test saves, refuses `load` without the provider and with a different name, reloads with the right one, replays identical turns and history, and continues identically. The 15 `stack_grounded_session` tests pass. Evidence branch: `codex/ci/log-recall-reload-main`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5946909128 — 2026-10-02T06:46:04Z — Casey-allard

**Exact-head review: `c7017298ff63f6bf9debf685a744a0bf018c1c6c`** (self-review, under the owner rule of 30 September on #820).

**Scope checked.**
- `SessionRecord.log_recall` is optional with `serde(default)`, so sessions saved before this change still load.
- `load` is `load_with_log_recall(.., None)` and refuses a session that recorded a provider. The provider is attached before `validate_history` replays the saved turns.
- A mismatch in either direction is a `Binding` error.
- `m-world session` reloads with its configured provider.

**Evidence at this head:** fmt and the check of all targets succeeded; `uor-r4-training` lib 527 passed, 0 failed; `m-world` examples pass. The PR merges cleanly with main.

**Measured:** best configuration reload 20/20 equal, against mostly unequal before the fix (#1552).

Verdict: approve for merge at this head.


---
_Comment/review ids in this file: 5946909128_

