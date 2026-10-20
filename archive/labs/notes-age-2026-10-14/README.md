# A floor with no age is not a warning

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** PM queue item 2 (unread-note surfacing)

## The gap, from this lab's own failure

The check-in already surfaces unread notes, deliberately from the **cache** so a check-in stays instant, and
`notes::render` already refuses to overstate - a cached read says **"at least N unread"**, because *reporting a
floor as a fact is worse than reporting it as a floor*.

**What was missing is the age.** The cache is only as fresh as the last time someone ran `notes`, and the
check-in line carried no indication of how old it was. This morning that cost this lab a tasking: at check-in
it printed `at least 1 unread note(s) on #2034` and I read past it, while **887 unread notes - including the
PM's 14:55 tasking - sat behind a cache hours old**. And a lab that had never run `notes` got **nothing at
all**, so it had no way to know there was anything to run.

## The change

The cache file's **mtime is the refresh time**, so no schema change is needed and existing caches keep working.
`cache_age` reads it, `cache_line` turns it into one line, and the check-in prints it:

```
uor-harness: notes were last refreshed 6h ago, so the count above is a floor - run: uor-harness notes
uor-harness: no notes cache on this machine yet - run: uor-harness notes
```

**The threshold is 20 minutes because that is the CONTEST window the PM named**, not a round number: past it,
a cache can already have missed a contest posted while the lab was working - which is the failure class this
closes. My first version used 30 minutes; tightening it to the named window is the difference between covering
the failure and covering a looser one.

**Still network-free.** Auto-fetching at check-in would break the property that a check-in is instant, which is
the entire reason the cache exists. The line reports; `notes` refreshes.

## Verified by running it, in all four cases

| case | result |
|---|---|
| cache 6h old | **`notes were last refreshed 6h ago …`** |
| cache 20m old (21m too) | reports - the boundary is the CONTEST window |
| cache fresh, or 19m | **silent** |
| no cache at all | **`no notes cache on this machine yet …`** (was silent before) |
| `--no-notes` | silent, as documented |

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | **500 passed, 0 failed, 6 ignored** |
| `cargo test --lib notes::cache_line_tests` | 5 passed |
| fmt, clippy `-D warnings` | clean |
| unittest, `parity.py`, interlock | exit 0, exit 0, 1641 = baseline |

## Stated rather than hidden

* **Nothing is fetched at check-in.** A lab that ignores the line still sees a stale floor; the line makes that
  visible rather than silently true.
* **The age is the cache file's mtime**, so it measures when `notes` last wrote - not when GitHub was last
  reachable. A failed refresh leaves the old mtime, which is the honest answer: the data is that old.
* **The threshold is a judgement, stated as one.** Twenty minutes is the CONTEST window; a lab that wants
  fresher data runs `notes`.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

This closes the PM's harness queue: claims store (#2472), transport retry (#2475), PATH install (#2476), and this.
