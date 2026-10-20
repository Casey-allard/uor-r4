# #1287 — Per-token compute counted: the readout is 99.97%; a 480x shortlist projection

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-19T20:27:25Z
- **Updated:** 2026-09-19T20:42:53Z
- **Closed:** 2026-09-19T20:28:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1287
- **Merged:** 2026-09-19T20:28:27Z
- **Merge commit:** dc96979cb7fbc58e6e9ba72530ee13d69f6ad552
- **Base / Head:** main / UOR-Foundation:codex/compute-breakdown
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Where the per-token compute actually goes

Your compute questions — can hyperbolic geometry or a deeper routing hierarchy save **massive** compute? — needed to know where the compute *is*, which had never been counted. Counted for the served path in its served configuration (`order = 2`, `vocab = 4096`, `dv = 64`):

```
address=2   read_exact=64   read_graded=7,680   readout=262,144
readout share of per-token work: 99.9748% (exact read) | 97.1530% (graded read)
```

**The read — the entire point of the geometric mechanism — is 0.03% to 2.8% of the work.** The vocabulary read dominates by three to four orders of magnitude.

**Projection from counted operations (not a built system):** a fanout-8, depth-4 route with a shortlist of `k = 8` costs `8·64 + 8·4 + 2 = 546` ops/token against 262,210 — **480× fewer operations**.

## Why the accuracy half is not settled, and is labelled everywhere as unsettled

On the synthetic relational task the **address determines the answer**, so a shortlist keyed on the address is trivially exact and the trade-off vanishes. The shortlist's real cost — generalisation lost when candidates are pruned — needs data where the address is not the answer's identity, i.e. **the real-text path**. So the 480× is a projection with an **unmeasured accuracy cost**, and it says so in the receipt, the test, `current-state.md` and `EVIDENCE.md`.

## What this changes

- The compute lever is **the readout** — consistent with the accuracy side of this session, where five readout levers were flat. The readout is simultaneously the cost bottleneck *and* not the accuracy bottleneck.
- **I3 ("candidates scored per token ≪ vocabulary") is where the large factors live.** The old artifact path already has a 64-candidate shortlist; the new geometric core has none — that is the gap.
- The hierarchy proposal survives in its useful form: **bounding the readout**, not hyperbolic arithmetic (which costs *more* per operation).

## A seventh would-be defect, caught before it was quoted

A first wall-clock comparison gave 264 µs (readout) vs 33 µs (write+read) — which looks like support, and isn't. The write+read side was dominated by **zeroing a 3.7 MB state buffer** (14400 × 64 ints), not by the read. That comparison measured allocation. Op counts are build-independent; wall-clock is not, so wall-clock is not used as evidence here.

**Tests:** `geometric_attention` **28/28**; `cargo fmt --check` clean; working tree clean. Nothing regressed.

**Ledger:** cumulative **146,178,565 / 147,200,000 ms** — **~17.0 minutes of model time remaining**. The next block needs a decision from you (below).

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

