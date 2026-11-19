# [codex] Restricting role candidates to the question recovers a minority of assertion intrusions

The frozen role banks recovered 28 of 140 named assertion-intrusion prompts per seed. The registered condition requires a strict majority. All 28 recoveries occur on held lowercase “ignoring” questions. The same mask recovered zero requested endpoints on fresh. The 192 held “Regarding” inversions, 192 fresh “Cedar” nominations and 192 fresh “Set aside” failures remain after the whole-question oracle removes the earlier assertions. The registered evidence therefore does not support learning this boundary as the next role repair. [Registration](SHEET.md) · [FIT](FIT.md)

Each seed independently gives the same counts below. The seeds share the case set and their counts are reported per question.

| Endpoint or paired measure | Held original | Held question | Fresh original | Fresh question |
|---|---:|---:|---:|---:|
| Exact requested out of 768 defined | 528 | 556 | 384 | 384 |
| Exact excluded out of 576 defined | 384 | 384 | 192 | 192 |
| Both requested endpoints exact in both directions out of 384 pairs | 264 | 268 | 192 | 192 |
| Abstention out of 288 with no requested referent | 144 | 168 | 124 | 144 |
| Absent requested winner among 768 explicit requests | 140 | 188 | 128 | 192 |
| Requested word elected as excluded out of 768 explicit requests | 192 | 192 | 0 | 0 |

All 528 held and 384 fresh baseline exact requests survived. All 384 held and 192 fresh exact excluded endpoints survived. Held role direction zero gained 24 and direction one gained four, with no losses. The held lowercase total rose from 240 to 268 out of 384 and the title-case total held at 288. Both fresh case totals held at 192 out of 384. Every recovered row came from the pre-sealed assertion cohort. That cohort had 76 held members, of which 28 recovered, and 64 fresh members, of which zero recovered. The shifted control elected zero exact requested and zero exact excluded on either panel. Its role decisions establish an adverse region can cut off otherwise correct endpoints.

## Where a request is lost

| Construction | Exact requested before and after | First rejection on the example |
|---|---:|---|
| Held delayed “excluding” | 192 → 192 | Exact role survives all 192 prompts. |
| Held delayed “ignoring” | 144 → 172 | In the lowercase May example, the learned request ranks second at score 28 behind “held” at score 38. The region removes “held” and May wins at the same score 28, from the original span `[16,18)`. |
| Held “Regarding” | 0 → 0 | Lowercase May at `[19,21)` reaches the excluded role at rank one and score 40 with either eligibility set. Its request rank is absent because the excluded score wins before requested ranking. |
| Held “belongs” | 192 → 192 | Both exact roles survive all 192 prompts. |
| Fresh “Set aside” | 0 → 0 | Sana at `[38,41)` moves from requested rank five to rank three and remains at score −16. The best surviving requested score in that example is −2 against a threshold of 12. |
| Fresh two-clause “Skip” | 0 → 0 | The true region includes both clauses. Cedar at `[28,31)` wins at score 36. Eve at `[22,24)` enters neither role rank. |
| Fresh “Except for” and “even though” | 384 → 384 | The original requested winners remain inside the complete question. |

Held errors under the mask comprise 188 no-winner rows, 20 nominations of “keep” and four nominations of “Regarding.” The exact requested endpoint was routed to the excluded slot on all 192 “Regarding” rows, including the four with a different requested winner. Fresh errors comprise 192 no-winner rows below threshold and 192 wrong-entity nominations of Cedar. The rank audit derives each annotated span's original ordinal, tie group and score by applying the same role operator to the candidate's own complete interval, preserving feature reads on the full prefix. Rows with equal role scores enter neither rank.

Held ordinary questions still make a false nomination on all 96 rows. The 24 additional abstentions all come from the 192 held unspecified-person requests. Fresh ordinary questions still make a false nomination on all 96 rows. The 20 additional abstentions all come from the 192 fresh unmentioned-person requests. A role nomination precedes the separately measured source, permission and writer transitions.

## Region provenance and adversarial controls

The generator emits a separate receipt at the byte boundary between its assertion rendering and its question rendering. The receipt binds the untouched original BOS-inclusive IDs. Its three decoded slices must exactly equal the complete assertion and user prefix, the entire question with its preceding space, and any annotated post-query text plus the final assistant marker. The generator checks all requested and excluded spans inside that interval. The rank audit separately verifies they are complete candidate spans under the frozen bank. Both tokenizer regenerations matched all six sealed TRAIN, held and fresh blind and grade files byte-for-byte. The published held and fresh fixtures contain no appended post-query sentences. The “Skip” instruction, the following “Report” clause and Cedar stay in the same question. The embedded Cedar after “even if” and Summit after “even though” also stay inside the question.

The adverse mask takes the equal-width interval immediately to the left of the question, clipped at BOS. Its end is the true interval's start. The named assertion cohort includes prompts with an annotated requested span where the original nominee lies wholly in the preceding rendered statement body, after the BOS/user marker. The adverse intervals and cohort were generated and hashed from predecessor output before the new arms ran. The requested-role oracle overrides only the original requested endpoints, including absent and conflicting support. It retains the original learnt exclusion and score fields as context for the intervention.

The separate 12-row nuisance fixture includes both directions and cases of four explicit requests and two no-referent questions, paired with the same prompts containing a post-query sentence. The true interval leaves the original question IDs in place and excludes the sentence “The margin reads: Report the thing for …”. The original and true-region arms yield eight exact requested and eight exact excluded out of eight defined endpoints, including both versions of each request. In one title-case no-referent prompt, the original arm nominates Joe `[44,45)` from the added sentence at score 34. The region removes that candidate and nominates “and” `[19,20)` at score 18 from the question. All four no-referent rows still make false role nominations. This finite witness verifies the boundary plumbing independently of the primary denominators.

## Raw replay and preservation

The original codex-12 executable reproduced all four held/fresh JSON decision files exactly. The new all-interval interface matched the original on all six split and seed combinations. Each pass compared 135,936 CUDA and Rust decision words across original, identity, region and adverse, including both span endpoints and signed winner and runner-up words. Whole multipart intervals, partial ends, empty intervals, invalid coordinates, the unchanged full-prefix reads and cross-seed refusal have finite checks. The cold repeat independently reloaded both frozen banks and re-emitted 30 decision files, with all role, grade and rank manifests matching byte-for-byte. The shared-lock collection took 5.84 seconds and the cold repeat 5.81 seconds on Wick's RTX 4090. Peak host memory was 160,648 KiB. No role fitting occurred.

The archive includes both seed banks and decision passes, the sealed inputs and region receipts, the named cohort, full pairwise grades, literal rank audit and executable receipts. It was copied from Wick and read back from the pinned store revision. [Raw archive](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/30b4e04f676c22ee213e2f0561d865bce04af39d/results/codex/query-region-pick-20261118/region-wick.tar.gz) · [Digests](RECEIPTS.txt) · [Wick runner](run-wick.sh) · [Role rank audit](rank/rank.rs)
