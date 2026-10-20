# #327 — Decision: adopt 300k-token corpus as next pinned era (+1.8pp measured)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-01T03:45:14Z
- **Updated:** 2026-08-01T21:07:46Z
- **Closed:** 2026-08-01T21:07:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/327
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Proposal: adopt a 300k-token corpus as the next pinned era (maintainer decision)

### Evidence (measured 2026-07-31, full matrix in #318)

Doubling the teacher corpus (150k → 300k tokens; first 150k byte-identical — the resumable stream extends cleanly) yields, within-run:

- **Shipped A row: 30.5/34.0/9.01 → 32.3/35.8/7.90** (+1.8pp top-1, +1.8pp agreement, −1.1 bits WB)
- **f32 ceiling: 32.1/36.4 → 34.6/38.9** (+2.5pp — the data curve is not flat)
- Phase B's residual margin widens from +0.5/+1.0 to **+1.4/+2.1pp** at the new scale
- The po2 norm fold's penalty shrinks to −0.1/−0.1, clearing its design-note gate without the mantissa refinement
- Marginal cost: ~8 minutes of gen per 150k tokens at 307 tok/s (post-#316)

This is the largest single accuracy lever measured to date — larger than every assignment-side improvement combined.

### What adoption means

1. Generate the 300k corpus on a maintainer machine (macOS-pinned, per Gate E discipline), verify 3× reproduction of the artifact κ.
2. Re-pin fixtures: `tests/fixtures/c_meta.bin` / `c_recs.bin` (+ hidden), `baseline_kappa.json` regeneration via `dump_baseline_kappa`, era notes per AGENTS.md.
3. Update era-sensitive expectations (`src/tless_uor.rs` resolution depths, PROOF.md §P2 matrix, BASELINE.md).
4. Re-scope #318 Phase B to the 300k era (crude norm fold ships; revisit cpy8-vs-cpy16 at +0.2pp).

### Open questions for the decision

- 300k or straight to a larger target (500k/1M)? The curve is unflattened at 300k; one more point (500k) would establish the scaling exponent before pinning. Gen cost is no longer the constraint (~15 min per 250k); fixture size and CI runtime are.
- Relationship to #320 (teacher upgrade): scaling the stories15M corpus first is the cheap rehearsal of the same re-pin machinery the teacher migration will need.

Origin: #310 review conversation, levers discussion; evidence run on `issue-318-dot-resid` tooling.


## Comments (1)

### comment 5152700789 — 2026-08-01T18:04:28Z — auser

## Three-point scaling curve (measured 2026-07-31; pinned 150k corpus restored intact; 500k corpus preserved at /tmp/corpus_500k)

| Corpus | A (shipped) | A-f32 ceiling | cpy8-resid (best buildable) | WB (shipped) |
|---|---|---|---|---|
| 150k | 30.5 / 34.0 | 32.1 / 36.4 | 31.1 / 35.1 | 9.01 |
| 300k | 32.3 / 35.8 | 34.6 / 38.9 | 33.7 / 37.9 | 7.90 |
| 500k | **33.4 / 37.3** | **35.7 / 40.3** | **35.3 / 39.6** | **7.15** |

**The curve is approximately log-linear, no knee**: shipped top-1 +1.8pp for the first doubling, +1.1pp for the next 0.74 doublings (log prediction +1.3). Per doubling ≈ +1.5-1.8pp. Milestone: at 500k the shipped integer runtime (33.4/37.3) already beats the 150k f32 ceiling (32.1/36.4).

**Recommendation: pin 500k.** It is preserved and ready (`/tmp/corpus_500k`, 2,507 stories); 1M is a plausible later era (~+0.8-1.0pp by extrapolation) but era bumps are cheap once the re-pin machinery is rehearsed — take the win now. Re-pin checklist per this issue's description (fixtures, `baseline_kappa.json`, era notes, `tless_uor.rs` depths, PROOF.md/BASELINE.md).

Era-relevant Phase B notes from the 500k run:

- **i8 centroid copies confirmed at every scale** (cpy8 ≡ cpy16; cpy8 is the best buildable row outright at 500k: 35.3/39.6).
- **Crude norm fold penalty is erratic across eras** (−0.4/−0.5 at 150k, −0.1/−0.1 at 300k, −0.7/−0.8 at 500k): the 1.5× mantissa-bit refinement is a live Phase C candidate at the pinned era.
- Phase B's residual margin at 500k: **+1.9/+2.3pp** over the shipped shape (35.3/39.6 vs 33.4/37.3) — the two levers keep compounding.
- Gen cost: 350k new tokens in 19 minutes (~305 tok/s, post-#316).



---
_Comment/review ids in this file: 5152700789_

