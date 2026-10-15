# #1778 — Factor geometric source relevance across typed Copy/Period/Stop actions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T02:37:18Z
- **Updated:** 2026-10-09T13:17:31Z
- **Closed:** 2026-10-07T00:07:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1778
- **Merged:** not merged
- **Merge commit:** 57235d59ca9f7a8b1985d44e321164654a645075
- **Base / Head:** codex/geometric-cue-coadapt-probe-20261006 / UOR-Foundation:codex/geometric-source-bound-actions-20261006
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

A geometric cue currently changes a source's Copy relevance while leaving its Period/Stop relevance unchanged; legacy terminals also use one hard-selected source. This adds an opt-in native integer path that scores endpoints for every retained source and can share the authentic cue across Copy/Period/Stop. Legacy remains the default.

The zero-fit development512 probe scores complete own-prefix generated replies: legacy74, all-source without terminal cue69, and matched all-source with shared cue72. Shared cue gains5/loses2 against the matched layout, but gains18/loses20 against legacy. Native CE improves0.95442→0.87141→0.86532; completion is primary, so no adoption or fresh-transfer claim follows. All arms and row tradeoffs are retained. No predictions on evaluation128, gradients or selection. DeepSeek's training-normalizer seam is untouched.

The bounded384-action layout preserves source/record/commit/event provenance and public256-context admission. The last real Stop uses the null slot without a phantom action. Flat normalization still has source phase-partition/length and terminal-alias effects; initial raw Copy rankings at a matched prefix are unchanged. This is an emission/relevance diagnostic, not a source-ranking or general-chat solution.

Validation at numerical head4cc70f08e2a3062b122d3348d8a5885a33e77a80 on Linux x86_64: formatting,34 source tests,4 reader tests,49 fitter tests and both release builds PASS (87 tests;229.463s). Model probe exit0 in39.375s,CPU16/RSS6.78GB. No CUDA/Metal or full-path allocation/opcode audit. Earlier fixture/admission failures are retained; final public API preserves EmptyBank rejection. Independent exact-file source reviews and saved-output reviews are recorded in the research evidence.

Durable sealed results: /workspace/uor-r4/codex/source-bound-actions/probe-1. Executable c2d15d2c7e5771490b773bc032e622035b23f37fcc4b8e92db2ffa1263a551fb; config4b514a962b3e064eb1c0834ef2222da5ba5a695465bd2f5db2c84cd989f56318. The numerical head binds compute; subsequent documentation leaves it unchanged.

Owner correction after the result holds the cue-only record-credit follow-up. Next source/history review identifies the smallest existing learned geometric state/transport and native emission integration where causal token error changes the representation itself. Current joint-cue credit freezes roots and this realizer only supports Copy/Period/Stop; it cannot establish general Generate. No integrated-learning execution is claimed.

Stacked on PR1777; not delivered to main. References #820.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6008224557 — 2026-10-06T02:40:40Z — Casey-allard

First exact-head check at9c26ee4b: formatting PASS; source tests33PASS/1FAIL. Failure was the new fixture counting originalstore tokens instead of the compiled emission view leading-space occurrence (expected8actions, actual10). Fixed only that assertion/comment atc5a1092e4c9260f4364d6be04301c1f53b06aa3f; runtime numerical code unchanged. Failed checks-1.json preserved under /workspace/uor-r4/codex/source-bound-actions. Full declared scoped checks are now running at the corrected head; no model result yet. Independent source review receipts bind9c26 reviewed runtime files; self-review confirms c5a changes only fixture expectation.

### comment 6027854538 — 2026-10-07T00:07:35Z — Casey-allard

Superseded by #1810, #1812 and #1813, all now merged through the protected queue. The selected active native runtime/CUDA/learning dependency closure is on main. Historical experiment drivers and unused API variants remain on this pushed branch as retained evidence; no branch is deleted and no mechanism-family verdict follows. Exact-head checks and preservation receipts: #1813 and #820 (triage issuecomment-6026806242).


---
_Comment/review ids in this file: 6008224557, 6027854538_

