# #453 — fix(compiler): #451 — canonical tie-break in emit_r4g1 (LIVE nondeterminism, not latent — needs a call)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T05:17:28Z
- **Updated:** 2026-10-09T14:16:49Z
- **Closed:** 2026-08-06T13:40:22Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/453
- **Merged:** 2026-08-06T13:40:22Z
- **Merge commit:** 275a4e1d70d5e0bd3efafc7fbce26c9412c1fb78
- **Base / Head:** i450-repro-hardening / UOR-Foundation:i451-canonical-tiebreak
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #451. **The issue understated this: it is not latent. Emitted `cover.r4g1` is nondeterministic TODAY.**

Evidence — three `transformerless cover` runs, identical inputs, pinned fixture, 79,348 bytes each:
- before: κ `e2f5d452…`, `e9286b83…`, `5087c271…` — three different digests
- after: κ `093a2dd4…` × 3

Two emissions **in a single process** already differed. Replaying old vs new selection across the fixture's 42 regions: 42/42 differ in write order, **15/42 differ in which tokens survive the 64-entry cut**, and 28/42 have a weight tie spanning the cut. The old selection's own set-wise digest differed between two runs in one process — the selected token *set* was nondeterministic in shipped output.

Fix: build in `BTreeMap` **and** sort `(Reverse(weight), token)`, matching the sibling transition compiler ~270 lines above which does exactly both. Sibling maps audited: `freq` fed only `candidate_weights`; `bigram_freq` accumulates a commutative integer sum — neither reached an ordering-sensitive decision, both converted anyway so the block is canonical by construction.

Regression test `emission_tie_at_truncation_cut_is_canonical` (non-ignored): 100-way tie across the cut, asserts 8 emissions byte-identical and canonical resolution. **Verified it fails on unfixed code** (exit 101).

**The decision, Casey:** no committed baseline is contradicted — every pinned κ addresses either the induced cover (untouched) or an R4G1 emitted from an empty observation slice. But this changes emitted `cover.r4g1` relative to any container already on disk, and 15/42 regions change tokens, which can move Gate C for runs that consume a pre-existing `--cover`. My read is that replacing random output with defined output is strictly an improvement and this should land — but it is your call, so it is stacked on #452 rather than merged.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

