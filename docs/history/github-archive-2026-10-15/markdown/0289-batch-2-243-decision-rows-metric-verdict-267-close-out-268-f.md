# #289 — batch-2: #243 decision rows (metric verdict), #267 close-out, #268 floor slices, #234 declaration, #276 design doc

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-29T22:39:31Z
- **Updated:** 2026-07-29T23:04:20Z
- **Closed:** 2026-07-29T22:50:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/289
- **Merged:** 2026-07-29T22:50:15Z
- **Merge commit:** c187ef03ddd2124175455ac38435010472fb0331
- **Base / Head:** main / UOR-Foundation:batch-2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Batch per the 2026-07-29 process amendment — one commit per issue-unit, gates run once here. Eight commits:

## #243 (three commits) — the Phase B decision, made by measurement
Round 1: A-R-retrained measured bit-identical to A-R — class_sigs already ARE centroid sign-signatures; recorded no-op. Round 2: A-dot-only 30.4/34.2 with no residual subtraction vs A-resid-sign collapsing to the floor — the dominant loss is the assignment metric, not residuals. Round 3: **A-dot-po2 30.6/34.2 and A-dot-po2x2 30.7/34.3 — both pass the Empirical Criterion (≥30.1/≥34.1)** at ~47-48k keys (7.2× under the shipped store). Phase B kernel design settled: shift-add dot against power-of-two centroids (shifts+adds only, contract §4). Kernel PR follows separately (runtime change = individual PR).

## #267 — closes on merge
The 'unre-materializable corpus' premise dissolved: manifest text_bytes was a character count; blake3 of the refetch equals the pinned corpus_cid bit-for-bit. Manifest gains text_chars/text_utf8_bytes/accounting note/retrieval field; fetch script verifies chars + CID; corpus uploaded as release asset d3-corpus-simple-wiki-20231101 keyed by corpus_cid.

## #268 — instrumentation (issue stays open for the replay fix)
evaluate-report (schema 2) decomposes the teacher floor by position-in-story band, next-token class, worst articles. First run named the dominant cause: record-stream story interleaving (~9.2-token runs) + per-run oracle resets → ~4.6-token mean effective context → the 13.4-bit floor is a bound under those conditions, not a model-quality bound (posted on the issue with the fix path).

## #234 — declaration machinery (item 2 stays open)
score_report schema 9 adds DistributionDeclaration: EXCT-miss rate fixed at declaration time + Gate C validity verdict against a declared 5% minimum, printed loudly at score time.

## #276 — Design P doc (REVIEW)
docs/phase_clock_address_design.md — cyclic graded codes on the zeta torus, positioned post-round-2 (channel addition, not residual-fix competitor); ⚑ flags open for Casey/Ari/Alex.

## Verification
cargo check/clippy(-D warnings)/fmt clean per commit on the warm shared target; three full certify runs + one evaluate-report run executed against these rows (logs on Casey's Mac); claim-wording gate passed. Full suite gates here in CI per the amendment.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018cWTa3B7xnsk6wygnfgL7o

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

