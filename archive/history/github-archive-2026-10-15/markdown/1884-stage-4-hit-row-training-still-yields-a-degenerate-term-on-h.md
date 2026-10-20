# #1884 — Stage 4: hit-row training still yields a degenerate term; on hit rows the correction hurts (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T08:36:52Z
- **Updated:** 2026-10-09T13:20:57Z
- **Closed:** 2026-10-08T08:38:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1884
- **Merged:** 2026-10-08T08:38:32Z
- **Merge commit:** 6e33a751c8760bd697fee1ce8a2ad564420eef27
- **Base / Head:** main / UOR-Foundation:deepseek/lut4-hitrows
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this changes

The harness now trains **only on rows whose target is inside the shortlist**, and measures those rows separately. Rationale from the previous document: for rows with the target outside the shortlist every candidate is negative, whose least-squares optimum *is* the constant term that produced the retracted result. 57 of 204 fit positions (28 %) and 52 of 256 held-out positions (20 %) are hits.

## Result: the lever is necessary and not sufficient

`chat-v0-p2` train → held-out, top-64 shortlist, 600 steps, 26-bit features:

| | value |
|---|---|
| training loss | 0.252945 → **0.223337** (the term does learn on hit rows) |
| degenerate? | **YES** — `distinct_hard_outputs` 1, `chosen_gates [12 ×16, 0]`, `hard_bit_tally [16384, 0]` |
| all held-out rows | baseline 4.211070, soft 4.201949 (−0.009121), hard 4.191505 (−0.019565) |
| **hit rows only (52)** | baseline 2.444325, soft 2.500545, **delta +0.056219 — worse** |

So:

- The constant-shift problem is **not** fixed by the data lever: the served term is still degenerate, the all-rows "improvement" is again the uniform shortlist penalty, and the guard says so instead of reporting it as a result.
- On the **only** rows where re-ranking can help, the learned soft correction makes bits per byte **worse by 0.056**. Training on the right rows was necessary and is not sufficient.

## Standing conclusion for Stage 4 in this session

Three forms measured, none improves held-out bits per byte on real text:

| form | result |
|---|---|
| all-candidate correction, best of 1,056 two-input terms | +0.0000061 |
| shortlist re-ranking, constant penalty (degenerate) | −0.019565 — not a result |
| shortlist re-ranking trained on hit rows (degenerate) | all-rows −0.019565; **hit rows +0.056219** |

**The Stage 4 gate is not passed**, and the evidence no longer supports another insertion-form iteration at this scale. Remaining levers, in the order the measurements point:

1. **Data scale for the term** — 204 fit positions cannot train a 26-bit interaction; the honest next test trains where features are dense, not on 200 rows.
2. **A different decision to re-rank** — this term re-ranks candidates of a scorer that is already weak on this corpus (baseline 4.21 bits/byte), so the ceiling is the scorer, not the term.
3. **Representation and data for the base scorer** — where every measurement in this session has pointed.

Scope: one artifact, one corpus pair, 204–256 positions, debug build, no serving default changed; served form remains a truth-table read. The hit-row split is reported so the 20 % ceiling of any shortlist re-ranker is visible rather than implied. Evidence: `docs/evidence/native_geometric_lut4_hitrows_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

