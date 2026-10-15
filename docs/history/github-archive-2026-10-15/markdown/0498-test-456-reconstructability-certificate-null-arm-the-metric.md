# #498 — test(#456): reconstructability certificate null arm — the metric is sub-unigram

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T11:04:17Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-08-08T11:21:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/498
- **Merged:** 2026-08-08T11:21:32Z
- **Merge commit:** 8f915f8e6f42f704e9f439d6daefd003c848dcf2
- **Base / Head:** main / UOR-Foundation:fix/456-reconstruction-null-arm
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #456. Completes the one exit criterion left open after PR #462 (the measurement infrastructure): the **null arm** (K-2 mutation discipline), plus the pre-declared verdict — which the measurement turned into a recorded **negative**.

## What
- `cover_sweep::reconstruction_null` — scores a cover point's EXCT-disabled reconstruction twice on the same held-out slice: real emission tables, then the same tables with every region's ΔE list **deranged** (seeded, root prior untouched).
- `tests/reconstruction_null.rs` — `#[ignore]`d harness; `R4_RECON_NULL_HELD` caps the slice (both arms + floor share it, so the verdict is scale-robust).

## Measured (default 42-region cover, 20k held-out of the 500k fixture; deterministic)
| arm | top-1 | bits/token |
|---|---:|---:|
| real emission tables | 0.0148 | 16.3209 |
| deranged (null) | 0.0034 | 19.3144 |
| unigram floor | **0.0636** | **8.7059** |

## Two findings
1. **The mutation guard passes.** Deranging the emission tables degrades the reconstruction by ~3 bits/token — a shape-matched shuffle does not preserve the score, so the metric reflects real region structure, not byte-shape.
2. **The reconstruction is sub-unigram — the pre-registered wording rested on a false premise.** "Shuffled must score *at* the unigram floor" assumed the real reconstruction sits above it. It does not: the EXCT-disabled graph reconstruction (16.3 bits / 1.5% top-1) is **7.6 bits worse and 4.9pp lower top-1 than a trivial unigram prior** (8.7 / 6.4%). This matches the repo's standing "graph-resolved ~1% top-1 / 16.3 bits" record — the graph residuals, applied without exact-context gating, are net-worse than unigram.

## Consequence (recorded negative)
`cover_sweep`'s recon-bits metric is a valid *relative* discriminator among covers, but every cover it ranks is sub-unigram on the graph-only path. Optimizing it as a compiler split/stop criterion (**item 3**) would optimize a quantity that never beats a trivial baseline — the same shape as the programme's standing pattern (resolution levers don't pay). Item 3 is **not** worth building on this metric as it stands; a tripwire assertion fires if a future change ever makes the graph reconstruction beat the floor. Record: `docs/reconstruction_null_456.md`.

## Also (source of truth)
Reconciles `docs/RESEARCH.md`'s "Open, with defined work" table with GitHub: drops the merged #487/#490 rows, corrects the stale #434 VSA wiring-gap claim into a #496 row (per #493's measurement), refreshes #488/#456, adds #457/#496.

fmt / clippy `--all-targets --all-features -D warnings` / claim-wording gate all clean. The `#[ignore]`d harness does not affect the default test run.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

