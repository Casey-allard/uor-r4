# [codex] Crossed endpoint rank attribution

Lowering the excluded score floor of 12 could recover **zero of 16** excluded people in any arm. In each arm, four complete excluded spans were absent and the twelve present spans failed the strict `excluded > requested` admission gate. The twelve present spans never entered the excluded ranking. No requested target was rejected by an admitted rival or a tie once it entered the requested rank. This was a post-score read of the exposed D30 panel. No thresholds or model inputs were changed.

| Scope and candidate iterator | Requested absent | Requested outside fence | Requested selected | Excluded absent | Excluded fails contrast |
|---|---:|---:|---:|---:|---:|
| Historical and inherited | 4 | 8 | 4 | 4 | 12 |
| Historical and repeated | 4 | 8 | 4 | 4 | 12 |
| Full and inherited | 4 | 0 | 12 | 4 | 12 |
| Full and repeated | 4 | 0 | 12 | 4 | 12 |

The full/repeated requested winners led the closest admitted rival by 20 to 70. For the twelve present excluded targets, raw excluded minus requested was −52 to −24. Their raw excluded scores were 4 to 34 points below the floor and 44 to 78 points below the actual excluded winner. Each had zero to four other spans with an equal raw excluded score before admission. Those counts do not represent admitted ties.

## Packing row 30100

“Ignore the entry for Quentin.” supplies the complete excluded original span `[73,78)`. It has the same five IDs as the earlier “Quentin packed stone bowls.” head at `[30,35)`. The two original tapes contain `[223,51,87,380,272]`, the vocabulary pieces for a standalone leading blank followed by `Q`, `u`, `ent`, and `in`. The span is addressable as a half-open range. No token differs from the factual mention, and no punctuation is fused into either span.

The GO-bound site table marks 223 class 0 in both the inherited role iterator and the assertion reader. Each remaining piece is class 2 in both. The period at position 29 is a valid delimiter (ID 16), but the next token at 30 is not assertion START. Position 31 is continuation and is preceded by the blank ID, not by the delimiter. The frozen binder therefore has no Quentin head, and the repeated-head iterator has no head to reuse. The same class pattern accounts for all eight absent endpoints on packing rows 30100–30107, including Quentin as the requested person when the roles reverse in 30104–30107.

The full-user requested role picks Natalia `[60,64)` on 30100 with scores `[48,−22]` and contrast 70. The excluded selector instead picks the single token “ the” at `[69,70)`, with scores `[−10,58]` and excluded runner-up 16.

## Delivery row 30116

The complete Benedict target `[70,73)` is present. Its score pair `[32,−20]` rejects it from excluded admission and makes it the requested runner-up at contrast 52. The requested Camilla span `[59,62)` wins at 72 from `[56,−16]`. The excluded winner is “ is” at `[55,56)` with `[4,24]`, ahead of the excluded runner-up at 10. This row reaches the excluded admission defect even with both endpoints present.

## Evidence

The [per-row attribution](role-rank-attribution.json) records exact score pairs, captured best and runner-up scores, winning spans, rival margins and the floor-only check. The [read-only Rust attribution](../role-attribution/attribution.rs) accepts the sealed D30 key and closed ordinary capture root. It validates the carrier and prompt tapes and reconstructs all eight role words for each of the 64 crossed captures. It does not load or call a model. The primary and cold attribution bytes matched on Wick.

- Attribution SHA-256 `ab88ef23887a107f254550e1f913cda4d97a9a26a7148c41b5006594063a172e`
- Ordinary capture manifest SHA-256 `c1c75612157aa9706fe06f7eda7f17d7b870fb8028be11477b47b38567b2c2df`
- Mode-7 site and packet SHA-256 `d722cb260ced8b0113ff634c60a98e93f909b206e6ffee2b735cef073d0270b7`
