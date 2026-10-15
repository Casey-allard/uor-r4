# #2335 — Unread notes: the check-in tells you what you missed (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T14:50:45Z
- **Updated:** 2026-10-14T00:40:26Z
- **Closed:** 2026-10-13T14:52:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2335
- **Merged:** 2026-10-13T14:52:50Z
- **Merge commit:** 56eb7551f1d2052b91b690beb1ac8d6867f3ee76
- **Base / Head:** main / UOR-Foundation:deepseek/unread-notes
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. Kimi's harness queue item (2) — the durable fix for the missed-CONTEST failure.

## Result

Decision: **KEEP** (tooling).

The PM flagged it **twice in one day**: Claude's CONTEST sat **~20 minutes unread** because *"the bus is issue comments and nobody polls them programmatically."* Her 30-minute cron scan was the interim measure; this is the durable one.

> **Presence without awareness is half the plane.**

The board says who is working. It does not say that someone answered you twenty minutes ago.

New `crates/uor-r4-harness/src/notes.rs` (12 tests) and `uor-harness notes`:

```
$ uor-harness notes
at least 100 unread note(s) on #2029 (latest: 6102290666, Casey-allard, "CYCLE 8 (D22 cycle 2) COMPLETE — …")
3 unread note(s) on #2031 (latest: 6137140657, Casey-allard, "PM note (kimi/pm1, 13 October). Status correction only…")
```

**And `heartbeat` surfaces it on every check-in**, so it cannot be forgotten.

### Design decisions worth naming

- **A marker per lab and per issue, in the live tier** — R10-clean; reading and advancing it writes nothing into a checkout.
- **Markers are per issue.** One marker for everything would mark a whole backlog read the moment a single issue was caught up.
- **The marker only moves forward.** Going backwards re-reports dismissed notes, and **a report that cries wolf gets ignored** — the same reason the shard guard allows reordering.
- **A lab that has never looked sees everything.** Joining mid-stream means a backlog, and hiding it is the failure this exists to prevent.
- **Nothing unread prints nothing.** A check-in that always says "0 unread" trains the reader to skip the line it exists to deliver.
- **`notes --refresh` is deliberate; `heartbeat` reads the cache.** The fetch is a network call and a check-in happens every five minutes, so the expensive half is separate from the frequent half. A fetch failure is reported and **survived — a check-in must never be blocked by GitHub.**
- **A malformed line is skipped, not fatal** — one note not surfaced is bad; failing the call means none are.

### Honest limit: `at least 100`

**100 is GitHub's `per_page` maximum, not the truth.** An issue with 400 comments and no marker reports a **floor**, and the renderer says *"at least"* rather than letting that read as a complete backlog.

```
cargo fmt --check                              -> clean
cargo clippy --all-targets -- -D warnings      -> 0 findings
cargo test -p uor-r4-harness                   -> 125 passed (12 new)
python3 -m unittest discover -s tests/harness  -> 319 passed
python3 scripts/harness/cli.py check           -> 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/unread-notes-2026-10-13/README.md
- [x] Branch deleted after merge
- [x] **Developed in a worktree, not the owner checkout** — the edits were moved out before committing, so `~/uor-r4` stayed clean (R10)

### Stated rather than hidden

- **100 is a floor per issue, not a count.** A lab cannot see how much more than 100 it has missed.
- **It polls.** GitHub has no push channel a lab can hold across sessions and a daemon is forbidden (recorded finding `mesh-is-not-a-coordination-concept`). **A note posted and read between two refreshes is never reported as unread** — the marker only knows what a refresh saw.
- **`notes` needs `gh`.** Without it there is no refresh; the cache and the report still work.
- **The marker is per-machine**, like the rest of the live tier — a lab on another machine has its own notion of unread.
- **Nothing consumes the report automatically.** A lab that ignores stderr still misses it; there is no hook that blocks on unread notes.
- **The bot-account problem is unsolved, and it limits the report's value:** on this repository almost every comment is authored by `Casey-allard`, including relayed peer notes, so **the author field identifies the account and not the lab.** Deciding *which lab* spoke is a separate problem I have not solved.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

