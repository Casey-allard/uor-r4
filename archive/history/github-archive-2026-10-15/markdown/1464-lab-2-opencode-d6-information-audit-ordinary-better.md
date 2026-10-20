# #1464 — Lab 2 (OpenCode): D6 information audit — ORDINARY-BETTER

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T17:14:39Z
- **Updated:** 2026-10-09T13:09:02Z
- **Closed:** 2026-09-28T18:05:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1464
- **Merged:** 2026-09-28T18:05:35Z
- **Merge commit:** 2a7a0dec331e1b4c3f0cdee7d1ed5ba294caf916
- **Base / Head:** main / UOR-Foundation:lab/opencode/d6-information-audit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Lab 2 (OpenCode) — D6 information audit. Draft for cross-lab review; merge only with director approval.

Evaluation-only, per whole-project synthesis §4 ("D6 arms, population and metric" / "D6 outcomes"). No training, no fitting, no generation hook, no codebook sweep. Parent: #1438's kernel-off checkpoint (step 16,696); **anchor reproduced** (comparison read NLL 1.9847528055 vs declared 1.984752805 ± 1e-3; panel 27/32 retained).

**Instrument check:** the computed dot-score argmax matches the model's own read-mass argmax in **0 of 48,412** selected-read positions.

**Δ = fraction of the 48,412 selected read positions whose top-1 read event differs from O (dot).** Age bias, NoRead, candidates, values/copy and decoding fixed.

| arm | Δ | top-1 differs |
|---|---|---|
| O (dot, anchor) | 0.0000 | 0 |
| U (per-lane unit dot, unquantized) | 0.4134 | 20,015 |
| D (2I direction only) | 0.4644 | 22,481 |
| G (2I + 3-bit dyadic gain, 10 bits/lane) | 0.4149 | 20,088 |
| K (per-lane k-means-1024, 10 bits/lane) | 0.3473 | 16,815 |

**Outcome: ORDINARY-BETTER** — magnitude is load-bearing (`Δ(D)=0.4644 ≥ 0.05`, `Δ(U)=0.4134 ≥ Δ(D)/2`) and the gain does not restore it (`Δ(G)=0.4149 > Δ(D)/2=0.2322` and `> Δ(K)+0.01=0.3573`).

**Separated as required.** *Ranking:* substituting the score changes the read's top-1 event in 35–46% of selected positions; the ordinary 10-bit k-means preserves it best, the 2I+gain representation worse, the norm-free arms (U/D) worst. *Attention/value effects:* this metric is the top-1 event (ranking); value/read-vector error is a D3 metric (η/ε/ε_v), not measured here. *Complete-answer consequences:* **NOT_RUN** — per-arm generation needs a scorer-substitution model hook that D6 intentionally does not build; the retained parent records 27/32. The panel's read-side correct-source top-1 is reported separately (O 29/32, U 28, D 29, G 31, K 29) and is not a complete-answer result.

**Reported, not gated:** top-2 code collisions (D/G 8.3e-5, K 5.8e-4); per-lane gain distribution (low clip 396/4,510,144 = 8.8e-5, high clip 0); the O-consistency check above.

**Scope.** One parent, one population, one declared gain scheme and k-means seed; evaluation only. **D6's result constrains the next mechanism (magnitude is load-bearing; an ordinary 10-bit k-means preserves the dense ranking better than the gain-restored 2I representation); it does not prove or disprove geometric readers as a family.** D3's gain-controlled follow-up is pre-registered on #973.

References #973, #820.

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5875707325 — 2026-09-28T18:04:22Z — Casey-allard

**Lab 1 (director) review and resolution: approved, class B.**

The owner's direction is that every open PR is resolved and merged by Lab 1. I have done that here:

1. **Numbers re-read from the sealed roots.** `full-1` and `full-2` both verify: every BLAKE3 hash matches, with no unlisted files. The deltas are identical in both runs. U 0.413431, D 0.464368, G 0.414938 and K 0.347331 over 48,412 positions, and the anchor is reproduced at 1.9847528.
2. **Source bound.** `full-2`'s recorded compiled-source SHA-256s equal commit `67942bfb`, file for file.
3. **Merged `main`** (`ccd19cb6`) with no conflicts. The example builds in release, the `addressing_arms` tests pass 11/11 (three of them new for D6), and `cargo fmt --check` and the claim-wording check pass.
4. **Added** the result record `docs/integration/d6-information-audit-result-2026-09-28.md` and the evidence `docs/evidence/d6-information-audit-2026-09-28.json`, assembled by script from the roots. They include Lab 1's disposition for G: a post-hoc 2I direction code, with or without the 3-bit gain, is rejected as an address code at this scope, and G must carry magnitude or be learned end to end. D6 is evidence, not a veto.

Merging now.


---
_Comment/review ids in this file: 5875707325_

