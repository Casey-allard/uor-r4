# #416 — feat(format,certify): #399 — FWDA forward-anchor section + score_candidates_infill serving entry point

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-04T21:37:23Z
- **Updated:** 2026-10-09T13:09:06Z
- **Closed:** 2026-08-04T22:13:04Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/416
- **Merged:** 2026-08-04T22:13:04Z
- **Merge commit:** 163a4b09fe8d8574d99b462f84174b8085f07ada
- **Base / Head:** main / UOR-Foundation:i399-serving
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #399. Productionizes the measured forward-anchor channel (M2/B′ records: gated-self fused 40.0% / 45.0-vs-39.4 live) as a serving-side capability.

**FWDA section** (`FWA1`, optional — absent section = feature off, all existing artifacts stay valid): NGRAM-mirrored layout; context_len byte = lookahead distance 1..=3, key.0 = anchor token, key.1 = the row's full pre-truncation total; entries store raw counts (token, u32), cap 64 per row (highest count, tie lowest token), min row total 2. Built from the same construction split that feeds `compile_context_rows` and the store (anchor and predecessor both in-train), stride-4 infill protocol.

**Loading/quantization:** load-time `quantize_fwda` in the sanctioned compiler-side float block next to `quantize_exct` — `ScoreQ::from_logprob(ln((c+0.5)/(total+V·0.5)))`, absent floor same law at c=0, carried per-row. Serving fusion itself is integer-only saturating adds.

**`score_candidates_infill(sig, code, recent, next_anchor: Option<(token, distance)>)`:** runs `score_candidates_coded`, then product-law re-ranking when the anchor row is live — candidates gain their fwd residual, fwd-only tokens enter at root_floor + transition_offset + residual, canonical re-argmax. Post-witness serving-side re-ranking: the witness keeps the base outcome, replay covers the base path. `next_anchor: None` and absent-section paths are byte-identical to `score_candidates_coded` (asserted).

**Tests (4/4, non-ignored):** table law matches the Gate C build; exact row round-trip; fusion selection matches the f64 reference (flip case asserted, floor-entry integer value exact); off-paths byte-identical. fmt/clippy clean across format, certify, cli.

Next PR: the gated two-pass generation path (pass-1 draft anchors + ExactContext gate per B′).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

