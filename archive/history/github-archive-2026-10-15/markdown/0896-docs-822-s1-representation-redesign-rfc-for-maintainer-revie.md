# #896 — docs(#822): S1 representation-redesign RFC — for maintainer review

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T10:48:22Z
- **Updated:** 2026-10-09T13:09:05Z
- **Closed:** 2026-08-21T11:09:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/896
- **Merged:** 2026-08-21T11:09:17Z
- **Merge commit:** 6fc4614909d4e8be78f1e798250a565330f35a79
- **Base / Head:** main / UOR-Foundation:rfc-822-s1-redesign
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this is

The S1 representation-redesign RFC the REVISE verdict requires — **design only, review-gated: no code lands from this document, and nothing below starts until you approve a direction.** It answers the four measured falsifiers (suffix-locality of scoring AND generation; the squeezed content-evidence bag; singleton-polluted thin confidence; zero trajectory awareness) with a ranked direction set, while restating everything that does not move (the 20‰ causal floor, the ≥25‰ lowering planning bar per #886/#887, the #838 gates, the #841 corrective bar, P-4/allocation/determinism invariants).

## The bets, in recommended order

1. **Q1 (verification, half a day):** confirm what prefix length the recorded `t_argmax` labels consumed in the obs-shard producer — if the observation protocol itself was short-prefix, every ceiling below is reframed at the source.
2. **D1 — joint conditional keys:** learn `P(answer | content-token, suffix)` residuals against the suffix-conditional (the quantity the falsified #891 arm never represented — it subtracted the *global* marginal). Not a sixth arm: it changes the table's key space, which is the representation lever the verdict names. Cheap offline instrument in the #875/#891 harness mold, backed-off-mix fallback pre-declared, sparsity reported.
3. **D2 — region-conditional evidence:** the artifact's own collision-free semantic codes (#784) as the evidence key, granularity swept against the #784 continuation-convergence null. The "holographic" bet stated plainly; also the only direction that natively grounds S3 trajectory state.
4. **D3 — deeper observation** (expensive branch, gated behind Q1 + D1/D2 results): the compiled evidence can never exceed what the observation protocol recorded.

D4 (positional/skip-gram lanes) rides inside D1's bake-off as a comparison arm. S2's sanctioned calibrator re-entry gets its content-side confidence features from whichever direction survives — same frozen gates.

## Questions for you (answers on this PR are the approval)

- **Q1 first** — agreed?
- **Q2** — may the D2 instrument read compiled region assignments directly from the released artifact (read-only), or do you want an API surface first?
- **Q3** — adopt ≥25‰ off-serving lower bound as the pre-declared bar for *opening* a lowering track (floor itself unchanged at 20‰)?
- **Q4** — budget stance on D3's multi-day teacher-compute branch: cap now, or defer until D1/D2 evidence exists?

**Do not merge until you're satisfied** — auto-merge is deliberately NOT armed on this PR; edits from your review get pushed here.

References #822

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01J2R2qs9kfcYpsjUua1SYmi


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

