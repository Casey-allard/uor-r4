# Live-tier heartbeats: check in every 5 minutes without touching a checkout

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** Kimi's harness queue item (1); owner direction 13 October; R10; D27

## The design

One rule: **a coordination write never touches a checkout.**

```
LIVE TIER   ~/.local/share/uor-r4/harness/          shared by every lab on this machine
              presence/<lab>.json        <- heartbeat writes HERE ONLY
              ledger/<lab>-<month>.jsonl <- heartbeat appends HERE ONLY
              claims.json                <- claim/release mirror (already)

COMMITTED   docs/harness/ledger/<lab>-<month>.jsonl <- ledger-sync appends from live
            docs/harness/claims.json                <- claims
```

- **Every lab writes to the same live directory**, so a check-in is visible to all of them
  immediately — no merge, no PR, no branch.
- **`PRESENCE_TTL_MINUTES` is 5.** The board marks a lab STALE once it goes quiet for longer, so
  "nobody has heard from Claude in 10 minutes" is now a fact on the board rather than a 30-minute
  blind spot.
- **`uor-harness ledger-sync`** appends this lab's live receipts into its committed shard before a PR,
  so the record still reaches review. **Append-only, idempotent, scoped to one lab.**

## Why the ledger did not simply move out of git

It would have been the easy reading of "out-of-tree writes", and it would have broken three things:
receipts would be invisible to review, the shard guard (which compares the committed shard against
`origin/main`) would have nothing to check, and a lab's record would stop being delivered with its
work. The live tier is the **write path**; the committed shard stays the **authority**.

## Measured

Every command run from a worktree, before and after `git status`:

| command | checkout write |
|---|---|
| `heartbeat` | **none** — presence and receipt both land live |
| `ledger-sync` | the committed shard, **by design** — that is the delivery step |

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
claude     STALE (10m)   starling read 2/3
codex      STALE (12m)   Count11; M1 0/0, M2 saved437/512 opened0
deepseek   live          live-tier test
kimi       STALE (36m)   evaluation integrity 1/3
```

That is the system working as asked: three labs quiet for 10–36 minutes are **visible as quiet**,
which the 30-minute window concealed.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **113 passed** (7 new) + 1 doc-test |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `cli.py check` | 0 errors |
| `check_no_softmax.py` | OK — 1648, unchanged |

Tests cover the properties that matter: a receipt appends **live** and creates nothing in the
checkout; the live shard **only grows**; sync **never rewrites or drops** committed lines; sync is
**idempotent**; sync is **scoped to one lab**; and an unwritable live tier is an **error**, not a
silent success.

## The judgement call: an unwritable live tier now fails

The live tier is the only write path, so when it cannot be written there is **no record at all**. The
check-in now returns an error rather than reporting success.

This replaced a test asserting the opposite — written when a durable tier existed as a fallback.
**A check-in that says "checked in" while recording nothing is worse than one that refuses**, because
the lab believes it is visible when no other lab can see it.

## Limitations, stated

- **The Rust binary is the write path; Python is not updated.** `cli.py heartbeat` still writes the
  durable presence copy and a checkout receipt, and still uses a 30-minute window. **Labs must use
  `uor-harness heartbeat`** for this behaviour — the documented command changes with this PR, but a lab
  running the old Python path gets the old behaviour. That is the largest gap in this delivery.
- **Nothing enforces the five minutes.** The board reports staleness; no hook makes a lab check in. A
  lab that goes quiet is visible, not corrected.
- **A lab that never runs `ledger-sync` records nothing committed.** Its receipts live in the live
  tier and reach no PR. The live tier is not a durable store and is not backed up.
- **The live directory is per-machine.** Two labs on different machines do not share it, so they do not
  see each other's presence — the committed tier remains the only cross-machine record.
