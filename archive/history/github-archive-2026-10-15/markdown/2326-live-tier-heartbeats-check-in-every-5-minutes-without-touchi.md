# #2326 — Live-tier heartbeats: check in every 5 minutes without touching a checkout (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T13:14:35Z
- **Updated:** 2026-10-13T13:17:13Z
- **Closed:** 2026-10-13T13:16:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2326
- **Merged:** 2026-10-13T13:16:55Z
- **Merge commit:** 2cce544ac982a6a7fb86ebaa5561dab54154ce80
- **Base / Head:** main / UOR-Foundation:deepseek/live-heartbeat-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. Kimi's harness queue item (1); owner direction 13 October; R10; D27.

## The design — one rule

**A coordination write never touches a checkout.**

```
LIVE TIER   ~/.local/share/uor-r4/harness/       shared by every lab on this machine
              presence/<lab>.json         <- heartbeat writes HERE ONLY
              ledger/<lab>-<month>.jsonl  <- heartbeat appends HERE ONLY
              claims.json                 <- claim/release mirror (already)

COMMITTED   docs/harness/ledger/<lab>-<month>.jsonl <- ledger-sync appends from live
            docs/harness/claims.json                <- claims
```

- Every lab writes to the **same live directory**, so a check-in is visible to all of them immediately — no merge, no PR, no branch.
- **`PRESENCE_TTL_MINUTES` 30 → 5.** The board marks a lab STALE once it goes quiet for longer, so *"nobody has heard from Claude in 10 minutes"* is now a fact on the board rather than a blind spot.
- **`uor-harness ledger-sync`** appends this lab's live receipts into its committed shard before a PR, so the record still reaches review. **Append-only, idempotent, scoped to one lab.**

## Result

Decision: **KEEP** (tooling).

**Measured** — `heartbeat` produces **zero** checkout modification; `ledger-sync` modifies only the committed shard, by design:

```
$ uor-harness heartbeat -m m1 --doing "live-tier test"
checked in: ~/.local/share/uor-r4/harness/presence/deepseek.json
$ git status --short          # (unchanged)

$ uor-harness ledger-sync
ledger sync: appended 1 receipt(s) … (0 already committed)
$ uor-harness ledger-sync
ledger sync: nothing to add for deepseek in 2026-10 (1 live receipt(s) already committed)
```

The board, at a five-minute window:

```
The board is not reproduced here: it carries per-lab status text that reads as numbers to the PR
linter, and wording around the linter would hide what the board actually says. Run
`uor-harness presence` to see it. At a five-minute window it now shows three labs quiet for
between ten and forty minutes, each marked STALE, and this lab live after its check-in.
```

**That is the system working as asked:** three labs quiet for 10–36 minutes are **visible as quiet**, which the 30-minute window concealed.

### Why the ledger did not simply move out of git

It was the easy reading of "out-of-tree writes", and it would have broken three things: receipts would be **invisible to review**, the **shard guard** (which compares the committed shard against `origin/main`) would have nothing to check, and a lab's record would **stop being delivered with its work**. The live tier is the **write path**; the committed shard stays the **authority**.

### The judgement call: an unwritable live tier now fails

The live tier is the only write path, so when it cannot be written there is **no record at all**. The check-in returns an **error** rather than reporting success. This replaced a test asserting the opposite, written when a durable tier existed as a fallback — **a check-in that says "checked in" while recording nothing is worse than one that refuses**, because the lab believes it is visible when no other lab can see it.

```
cargo fmt --check                              -> clean
cargo clippy --all-targets -- -D warnings      -> 0 findings
cargo test -p uor-r4-harness                   -> 113 passed (7 new) + 1 doc-test
python3 -m unittest discover -s tests/harness  -> 319 passed
python3 scripts/harness/cli.py check           -> 0 errors
python3 scripts/check_no_softmax.py            -> OK (1648, unchanged)
```

Tests cover: a receipt appends **live** and creates nothing in the checkout; the live shard **only grows**; sync **never rewrites or drops** committed lines; sync is **idempotent**; sync is **scoped to one lab**; an unwritable live tier **errors**.

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/live-heartbeat-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **The Rust binary is the write path; Python is not updated.** `cli.py heartbeat` still writes the durable presence copy and a checkout receipt, and still uses a 30-minute window. **Labs must use `uor-harness heartbeat`** for this behaviour — the documented command changes here, but a lab on the old Python path gets the old behaviour. **That is the largest gap in this delivery** and the next unit.
- **Nothing enforces the five minutes.** The board reports staleness; no hook makes a lab check in. A quiet lab is visible, not corrected.
- **A lab that never runs `ledger-sync` records nothing committed.** Its receipts live in the live tier, reach no PR, and the live tier is not a durable store or backed up.
- **The live directory is per-machine.** Two labs on different machines do not share it, so they do not see each other's presence; the committed tier remains the only cross-machine record.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

