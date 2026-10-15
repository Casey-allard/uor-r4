# #2299 — Fix two false positives in the new claim check, and a 3.9 interop bug (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T05:24:51Z
- **Updated:** 2026-10-13T05:27:22Z
- **Closed:** 2026-10-13T05:27:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2299
- **Merged:** 2026-10-13T05:27:07Z
- **Merge commit:** 31ec5a519cc9af89fea3f8f6ddcf51e228cf8516
- **Base / Head:** main / UOR-Foundation:fix/claim-merged-race
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## Result

Decision: **KEEP** (tooling) — the rules stay, with the false positives removed.

## Two false positives on live work, and a 3.9 interop bug

The PM-proposed merged-claim check shipped an hour ago in #2296 and produced a **false positive on
live work within minutes**: `kimi/plan-refresh-2026-10-13` was claimed at 01:16 and flagged as
already merged in the same window, then the warning vanished once the push became visible.

**Cause:** the check asked *"is it gone from `origin`?"* **before** asking *"is there a merged PR?"*.
A branch pushed seconds ago can be in neither place yet, and a brand-new PR may not come back from
the API immediately, so the branch looked deleted → therefore merged.

A rule that cries wolf on a lab's live branch is **worse than no rule**: it teaches people to ignore
the check that exists to stop them colliding.

**Fixed by reversing the order.** A merged PR is the direct evidence and is asked first. When GitHub
says there is no merged PR and the branch *is* visible, that is unmerged work, full stop. When
neither can be established, the answer is "not established" and the rule stays silent.

```
kimi/plan-refresh-2026-10-13    -> False   (OPEN PR #2297; not flagged)
kimi/pm-rules-2026-10-13        -> True    (PR #2295 merged; flagged)
never/pushed-branch             -> None    (unknown; silent)
```

Three tests pin it, including the race itself.

### And a real Python 3.9 bug — found only because CI runs both

The suite **passed on 3.15 and failed on 3.9**:

```
FAIL: test_claim_timestamps_parse_in_both_offset_forms (text='2026-10-13T05:00:00Z')
AssertionError: unexpectedly None : 2026-10-13T05:00:00Z
```

`datetime.fromisoformat` did not accept the `Z` suffix until Python **3.11**, and this harness
supports **3.9**.

**This is not a compatibility nicety — it breaks the Rust interop this session was built around.**
The Rust implementation writes `Z` timestamps. Without this fix, a claim written in Rust is
unreadable by the Python harness on 3.9, and the staleness rule silently stops working: no error, no
warning, just a check that quietly does nothing. Both offset forms now parse on both interpreters.

```
302 tests pass on 3.15 AND 3.9
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md (the port this interop belongs to)
- [x] Branch deleted after merge

### Stated rather than hidden

- The race is **narrowed, not eliminated**. If GitHub returns neither a merged PR nor the branch, the
  rule stays silent — so a genuinely merged claim could go unflagged in that window. Silence is the
  right failure here: a missed warning costs a stale claim, a false one costs a lab's trust in the
  check.
- The 3.9 bug was live in the **committed** rules for about an hour. Any lab running `check` on
  Python 3.9 during that window got a silently non-functional staleness check.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

