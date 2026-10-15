# #2323 — Claims move to the caller's checkout: R10 and the shared store reconciled (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T12:45:56Z
- **Updated:** 2026-10-13T12:49:26Z
- **Closed:** 2026-10-13T12:49:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2323
- **Merged:** 2026-10-13T12:49:06Z
- **Merge commit:** 3c8ef2e85e1e9af413311f34b4dd7d63cb9bac2b
- **Base / Head:** main / UOR-Foundation:deepseek/claims-owned-20260713
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. Session task (3), owner decision **option 2**.

## The conflict

Two rules could not both hold:

- **R10** — `~/uor-r4` is **read-only for every lab**.
- **`claims_file()`** — resolved to `main_worktree()`, i.e. **the owner checkout**, because a claim had to land somewhere shared and committable.

The old docstring gave the measured reason for the second rule, and it was not a mistake: *two Kimi claims from 12 October died with their worktree while the shared store stayed empty.* But it meant **every `team claim` dirtied the owner's checkout — the tool itself violated R10.**

## Result

Decision: **KEEP** (tooling).

`claims_file()` now resolves to **the caller's own checkout**, and the claim reaches the other labs **through the delivery PR**, as receipts already do. `load_claims` merges three tiers with the **committed copy as the arbiter** and the live tier filling gaps.

### The first attempt was wrong, and the reason matters

Merging the tiers **newest-`claimed_at`-wins** failed because `claim` stamps **one entry into both tiers with a single timestamp**: they **tie immediately**, then **diverge** under a hand edit, a revert or a re-claim. "Newest" cannot choose between two claims that are both current. It silently picked live, so **a two-day-old claim kept blocking** — the exact opposite of what the staleness rule is for.

So the live tier is a **mirror, not a second authority**, published on `claim` **and on `release`**. A mirror only ever added to **resurrects released claims**, which is what made `release` look broken: the committed copy dropped the branch and the stale live copy read it back.

### Verified end-to-end on the real checkout

```
$ cd .worktrees/deepseek-claims-owned && cli.py team claim --branch deepseek/probe-claim --lab deepseek
claimed deepseek/probe-claim for lab deepseek
  head 841d15714 · …/.worktrees/deepseek-claims-owned/docs/harness/claims.json

$ cd ~/uor-r4 && git status --short
(empty)                                  <- R10 HOLDS
```

The claim lands in the caller's worktree, the owner's checkout is untouched, and the live mirror holds it for the other labs.

### Two bugs the tests caught

**1. Comparing ISO stamps as strings across UTC offsets.** The harness writes local Eastern (`-04:00`); a hand edit or another tool writes UTC (`+00:00`). `"2026-10-11T12:38:52+00:00"` sorts **after** `"2026-10-13T08:38:52-04:00"` although it is **two days earlier**, because the offset is never consulted. Now parsed before comparing, with `Z` normalised because `fromisoformat` rejects it before 3.11 and this file is imported on the declared 3.9 floor.

**2. `_tier` leaking into the committed store.** A read-time annotation written back as though it were part of the claim, annotating every existing entry. Stripped on write and pinned by a round-trip test.

### And one found by using the tool

`multilab.claim` mirrored correctly; **`team claim` — the command labs actually run — did not**, because it reaches the store through `receipts.claim_branch`. **The feature looked like it worked while the real path did nothing.**

```
python3 -m unittest                  -> 319 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check -> 48 rules, 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/claims-owned-tier-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

**A real regression, and it is the price of R10 holding:** two labs claiming the same branch in **different worktrees** both succeed locally. The collision surfaces when the second PR conflicts on `claims.json`, **not at claim time**. Against the old single shared file, that is a loss of immediacy — and I did not find a way to keep both properties without one lab writing another lab's checkout.

- **A claim is only durable once committed.** A lab that claims and never commits still loses it to worktree cleanup — narrowed to "claims that are never delivered" rather than "every claim".
- **`_tier` is provenance, not a field.** It is present only on claims read from a tier other than the caller's own, so a reader must not treat it as part of the claim.
- **Task (1) is untouched**: `status` section 4, `briefing`, `mistakes`, `prlint` and `check`'s rules remain Python.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

