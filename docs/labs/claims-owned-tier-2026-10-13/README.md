# Claims move to the caller's checkout — R10 and the shared store are reconciled

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** session task (3), pm-rules **R10**, owner decision 13 October (option 2)
**Delivered by:** PR #2322

## The conflict

Two rules could not both hold:

- **R10** — `~/uor-r4` (the owner checkout) is **read-only for every lab**.
- **`claims_file()`** — resolved to `main_worktree()`, i.e. **the owner checkout**, because a claim
  had to land somewhere shared and committable.

The old docstring gave the measured reason for the second rule, and it was not a mistake:

> A claim written into a worktree's own `docs/harness/claims.json` is invisible to every other lab,
> and the worktree — and with it the claim — is deleted by routine cleanup. That is measured, not
> theoretical: **two Kimi claims from 12 October left receipts in `kimi-2026-10.jsonl` while the shared
> store stayed empty.**

So every `team claim` dirtied the owner's checkout, and R10 was violated by the tool itself.

## The decision

Put to the owner with three options. **Chosen: option 2.**

> `claims_file()` resolves to the **caller's own worktree**, and the claim reaches the other labs
> through the delivery PR — the same mechanism receipts already use.

The old concern is **answered rather than ignored**: a claim is not lost to cleanup because it is
committed on the lab's branch before the worktree goes, and `multilab claims` merges the committed
store with the live tier and main's store, so a claim is visible from the moment it is made rather
than only after its PR merges.

## What changed

| | before | after |
|---|---|---|
| `claims_file()` | `main_worktree()` — the owner checkout | **the caller's own checkout** |
| `load_claims()` | one file | **three tiers**, committed authoritative |
| live tier | not used for claims | **mirror**, published on claim and release |
| `_tier` annotation | — | added on read, **stripped on write** |

### One arbiter per branch, and it is the committed copy

`load_claims` reads, weakest first:

1. **live** (`~/.local/share/uor-r4/harness/claims.json`) — real time, uncommitted, advisory.
2. **shared** — main's committed store, what the other labs have merged.
3. **the caller's own committed store** — authoritative for every branch it names.

The first attempt merged the tiers **newest-`claimed_at`-wins** and it was **wrong**, for a reason
worth recording: `claim` stamps one entry into both tiers with a single timestamp, so the copies
**tie immediately**, and then **diverge** under a hand edit, a revert, or a re-claim. "Newest" cannot
choose between two claims that are both current. It silently picked live, and a **two-day-old claim
kept blocking** — the exact opposite of what the staleness rule is for.

### The mirror is a mirror, not a second authority

The live tier exists so R10 can hold **without hiding work in progress**. It is published on `claim`
*and on* `release`: a mirror that is only ever added to **resurrects released claims**, which is what
made `release` appear broken — the committed copy dropped the branch and the stale live copy read it
back.

### A duplicate write path, found by using the tool

`multilab.claim` mirrored correctly; **`team claim` — the command labs actually run — did not**,
because it reaches the store through `receipts.claim_branch` instead. The feature looked like it
worked while the real path did nothing. Both paths now publish.

## Verification

**End-to-end, on the real checkout:**

```
$ cd .worktrees/deepseek-claims-owned
$ cli.py team claim --branch deepseek/probe-claim --lab deepseek
claimed deepseek/probe-claim for lab deepseek at 2026-10-13T08:43:11-04:00
  head 841d15714 · …/.worktrees/deepseek-claims-owned/docs/harness/claims.json

$ cd ~/uor-r4 && git status --short
(empty)                        <- R10 HOLDS
```

The claim lands in the caller's own checkout, the owner's checkout is untouched, and the live mirror
holds it for the other labs:

```
mirror claims: ['claude/readme-geometry-2026-10-13', 'deepseek/probe-claim', 'kimi/pm-rule-r10-workspaces-20261013']
```

**Tests: 319 passed on 3.15 AND 3.9** (four added: the mirror is not a second authority, a live claim
is visible uncommitted, a committed claim outranks a stale live copy, and release removes the mirror
entry).

## Two bugs the tests caught

**1. Comparing ISO strings across UTC offsets.** `_instant` parses before comparing, because the
harness writes local Eastern (`-04:00`) and a hand edit or another tool writes UTC (`+00:00`).
Comparing those as strings is wrong **and silently so**: `"2026-10-11T12:38:52+00:00"` sorts after
`"2026-10-13T08:38:52-04:00"` even though it is two days earlier, because the offset is never
consulted. `Z` is normalised because `fromisoformat` rejects it before Python 3.11 and this file is
imported on the declared 3.9 floor.

**2. `_tier` leaking into the committed store.** The annotation describes where a claim was *read
from*, not what it says, so writing it back annotated every existing claim as though the annotation
were part of the claim. Stripped on write, pinned by the round-trip leaving no residue.

## Limitations, stated

- **A claim is only durable once committed.** A lab that claims and never commits still loses it to
  worktree cleanup — the old failure mode, now narrowed to "claims that are never delivered" rather
  than "every claim".
- **Two labs claiming the same branch in different worktrees both succeed locally.** The collision is
  detected when the second PR conflicts on `claims.json`, not at claim time. That is a real regression
  in immediacy against the old single shared file, and it is the price of R10 holding.
- **`_tier` is a read-time annotation**, so it is absent from a claim the caller's store owns. A reader
  must treat it as provenance, not as a field.
- **Task (1) is untouched by this**: `status` section 4, `briefing`, `mistakes`, `prlint` and
  `check`'s rules remain Python.
